//! GPU-05 native owned Wayland surface, actual signed tetra/SDF shader, Vulkan WSI.
#[path = "../first_frame_barrier.rs"]
mod first_frame_barrier;
#[path = "../first_present_latency.rs"]
mod first_present_latency;
#[path = "../frame_pacing03.rs"]
mod frame_pacing03;
#[path = "../host_stall_probe.rs"]
mod host_stall_probe;
#[path = "../optical_probe.rs"]
mod optical_probe;
#[path = "../owned_wayland_presentation.rs"]
mod owned_wayland_presentation;
#[path = "../scene_bridge.rs"]
mod scene_bridge;
#[path = "../startup_policy.rs"]
mod startup_policy;
use blobin_cp01_hybrid::gltf::import_glb;
use first_frame_barrier::{FirstFrameBarrier, Outcome, StartupMode};
use first_present_latency::{calculate, monotonic_now_ns, StartupAnchors};
use frame_pacing03::FrameCallbackPacing;
use host_stall_probe::{thread_cpu_time_ns, HostStallProbe};
use optical_probe::{OpticalProbe, Transition};
use owned_wayland_presentation::OwnedGpuWindow;
use scene_bridge::SceneStream;
use startup_policy::StartupPolicy;
use std::{
    collections::BTreeMap,
    error::Error,
    fs::{File, OpenOptions},
    io::{BufWriter, Write},
    path::Path,
    sync::mpsc::Receiver,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use wgpu::util::DeviceExt;
const W: u32 = 640;
const H: u32 = 432;
struct Live {
    gpu: OwnedGpuWindow,
    config: wgpu::SurfaceConfiguration,
    device: wgpu::Device,
    queue: wgpu::Queue,
    compute: wgpu::ComputePipeline,
    compute_bind: wgpu::BindGroup,
    rgba: wgpu::Buffer,
    texture: wgpu::Texture,
    params: wgpu::Buffer,
    draw: wgpu::RenderPipeline,
    draw_bind: wgpu::BindGroup,
    optical_uniform: Option<wgpu::Buffer>,
    optical_probe: Option<OpticalProbe>,
    optical_trace: Option<BufWriter<File>>,
    optical_feedback_pending: BTreeMap<u64, (u64, bool)>,
    started: Instant,
    report_at: Instant,
    previous_submit_at: Option<Instant>,
    submit_intervals_ms: Vec<f64>,
    frames: u64,
    wsi_timeout_streak: u32,
    wsi_timeout_total: u64,
    wsi_occluded_total: u64,
    next_frame_sequence: u64,
    presented_frames: u64,
    discarded_frames: u64,
    first_frame_outcome: Option<Outcome>,
    first_submit_begin_mono_ns: Option<u128>,
    first_submit_return_mono_ns: Option<u128>,
    first_compositor_present: Option<(u64, u128)>,
    last_feedback_refresh_ns: Option<u32>,
    trace: Option<BufWriter<File>>,
    callback_trace: Option<BufWriter<File>>,
    stage_trace: Option<BufWriter<File>>,
    stage_origin: Instant,
    callback_sequences: Vec<u64>,
    callback_count: u64,
    token: String,
    yaw: f32,
    pause_started: Option<Instant>,
    paused_total: Duration,
    semantic_stream: Option<SceneStream>,
    update_rx: Option<Receiver<Vec<u8>>>,
    live_mode: bool,
}

impl Live {
    async fn create(
        stream: Option<SceneStream>,
        update_rx: Option<Receiver<Vec<u8>>>,
        trace: Option<BufWriter<File>>,
        callback_trace: Option<BufWriter<File>>,
        stage_trace: Option<BufWriter<File>>,
        output: Option<&str>,
        optical_trace: Option<BufWriter<File>>,
    ) -> Result<Self, Box<dyn Error>> {
        let mut desc = wgpu::InstanceDescriptor::new_without_display_handle();
        desc.backends = wgpu::Backends::VULKAN;
        let instance = wgpu::Instance::new(desc);
        let gpu = OwnedGpuWindow::open_on_probe(&instance, output, optical_trace.is_some())?;
        if let Some(src) = stream.as_ref() {
            println!("GPU05_SCENE_STREAM frames={}", src.frame_count());
        }
        let surface = gpu.wgpu_surface();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(surface),
                power_preference: wgpu::PowerPreference::HighPerformance,
                ..Default::default()
            })
            .await?;
        let info = adapter.get_info();
        if info.backend != wgpu::Backend::Vulkan || info.vendor != 0x10de {
            return Err(format!("GPU02_EXPECTED_REFERENCE_NVIDIA_VULKAN got={info:?}").into());
        }
        println!(
            "GPU02_PHYSICAL_ADAPTER={:?} {} driver={}",
            info.backend, info.name, info.driver
        );
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await?;
        let (width, height) = gpu.size();
        if width == 0 || height == 0 || width > 4096 || height > 4096 {
            return Err("GPU05_INITIAL_SURFACE_SIZE_OUT_OF_BOUNDS".into());
        }
        let mut config = surface
            .get_default_config(&adapter, width, height)
            .ok_or("no compatible Wayland surface format")?;
        config.present_mode = wgpu::PresentMode::AutoVsync;
        surface.configure(&device, &config);

        // The same imported, explicitly certified convex MeshSolid as GPU-01.
        let imported = import_glb(include_bytes!(
            "../../../CP01-HYBRID/fixtures/tetra-indexed.glb"
        ))
        .map_err(|e| format!("GLB_IMPORT_REJECTED {e:?}"))?;
        let solid = imported
            .try_certify_convex()
            .map_err(|e| format!("MESHSOLID_CERTIFICATION_REJECTED {e:?}"))?;
        let mut geo = Vec::<[f32; 4]>::new();
        for t in &solid.mesh().triangles {
            for idx in t {
                let p = solid.mesh().vertices[*idx as usize];
                geo.push([p.x as f32, p.y as f32, p.z as f32, 0.0]);
            }
        }
        for t in &solid.mesh().triangles {
            let [a, b, c] = t.map(|i| solid.mesh().vertices[i as usize]);
            let n = b.minus(a).cross(c.minus(a));
            let n = n.scale(1.0 / n.norm());
            geo.push([n.x as f32, n.y as f32, n.z as f32, (-n.dot(a)) as f32]);
        }
        if geo.len() != 16 {
            return Err("EXPECTED_FOUR_CERTIFIED_FACES".into());
        }
        let mesh = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("gpu02 certified GLB triangles"),
            contents: bytemuck::cast_slice(&geo),
            usage: wgpu::BufferUsages::STORAGE,
        });
        let samples = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("gpu02 unused field-sample placeholder"),
            contents: bytemuck::cast_slice(&[[0.0_f32; 4]]),
            usage: wgpu::BufferUsages::STORAGE,
        });
        let sample_out = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("gpu02 unused reference output"),
            size: 16,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        let rgba = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("gpu02 realtime GPU produced image"),
            size: (W * H * 4) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let params = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("gpu02 observation-only camera/time"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut entries = Vec::new();
        for b in 0..4 {
            entries.push(wgpu::BindGroupLayoutEntry {
                binding: b,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: b < 2 },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            });
        }
        entries.push(wgpu::BindGroupLayoutEntry {
            binding: 4,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("gpu02 compute bindings"),
            entries: &entries,
        });
        let compute_bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("gpu02 signed mesh compute"),
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: mesh.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: samples.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: sample_out.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: rgba.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: params.as_entire_binding(),
                },
            ],
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("gpu02 actual signed triangle compute"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../surface_live.wgsl").into()),
        });
        let compute_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("gpu02 compute layout"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let compute = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("gpu02 signed mesh raymarch"),
            layout: Some(&compute_layout),
            module: &shader,
            entry_point: Some("render"),
            compilation_options: Default::default(),
            cache: None,
        });
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("gpu02 GPU-only image transfer"),
            size: wgpu::Extent3d {
                width: W,
                height: H,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let texview = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("gpu02 linear sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let optical_uniform = if optical_trace.is_some() {
            Some(
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("oracle02 optical ROI only"),
                    contents: bytemuck::cast_slice(&[0.0f32; 4]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                }),
            )
        } else {
            None
        };
        let mut draw_entries = vec![
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ];
        if optical_uniform.is_some() {
            draw_entries.push(wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            });
        }
        let draw_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("gpu02 draw bindings"),
            entries: &draw_entries,
        });
        let mut bind_entries = vec![
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&texview),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ];
        if let Some(buffer) = optical_uniform.as_ref() {
            bind_entries.push(wgpu::BindGroupEntry {
                binding: 2,
                resource: buffer.as_entire_binding(),
            });
        }
        let draw_bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("gpu05 scene projection and optional optical ROI"),
            layout: &draw_layout,
            entries: &bind_entries,
        });
        let draw_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("gpu02 graphics presenter"),
            source: wgpu::ShaderSource::Wgsl(if optical_uniform.is_some() {
                include_str!("../present_optical.wgsl").into()
            } else {
                include_str!("../present.wgsl").into()
            }),
        });
        let draw_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("gpu02 draw pipeline layout"),
            bind_group_layouts: &[Some(&draw_layout)],
            immediate_size: 0,
        });
        let draw = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("gpu02 Wayland swapchain render"),
            layout: Some(&draw_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &draw_shader,
                entry_point: Some("vs"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &draw_shader,
                entry_point: Some("fs"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        println!("GPU05_HYBRID_REAL_WGSL_VULKAN no_png_intermediary=true");
        let token = format!(
            "gpu05-owned-{}-{}",
            std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
        );
        let mut trace = trace;
        if let Some(out) = trace.as_mut() {
            serde_json::to_writer(
                &mut *out,
                &serde_json::json!({
                    "type":"header","schema":"blobin.gpu05.presentation.v1",
                    "feedback_origin":"wp_presentation",
                    "feedback_surface_owner":"gpu05-owned-wayland-surface",
                    "wl_surface_token":token,"clock_id":gpu.clock(),
                }),
            )?;
            out.write_all(b"\n")?;
            out.flush()?;
        }
        let mut optical_trace = optical_trace;
        if let Some(out) = optical_trace.as_mut() {
            serde_json::to_writer(
                &mut *out,
                &serde_json::json!({
                    "type":"header","schema":"blobin.gpu05.optical.renderer.v1",
                    "wl_surface_token":token,
                    "input_origin":"wl_pointer_button_scoped_to_own_surface",
                    "roi":"uv_x_0.10_to_0.26_y_0.10_to_0.26",
                    "physical_capture":"NOT_RUN",
                    "semantic_authority":"unchanged_renderer_local_projection",
                }),
            )?;
            out.write_all(b"\n")?;
            out.flush()?;
        }
        println!("GPU05_OWNER token={} clock={:?}", token, gpu.clock());
        Ok(Self {
            gpu,
            config,
            device,
            queue,
            compute,
            compute_bind,
            rgba,
            texture,
            params,
            draw,
            draw_bind,
            optical_uniform,
            optical_probe: optical_trace.as_ref().map(|_| OpticalProbe::new()),
            optical_trace,
            optical_feedback_pending: BTreeMap::new(),
            started: Instant::now(),
            report_at: Instant::now(),
            previous_submit_at: None,
            submit_intervals_ms: Vec::with_capacity(2048),
            frames: 0,
            wsi_timeout_streak: 0,
            wsi_timeout_total: 0,
            wsi_occluded_total: 0,
            next_frame_sequence: 1,
            presented_frames: 0,
            discarded_frames: 0,
            first_frame_outcome: None,
            first_submit_begin_mono_ns: None,
            first_submit_return_mono_ns: None,
            first_compositor_present: None,
            last_feedback_refresh_ns: None,
            trace,
            callback_trace,
            stage_trace,
            stage_origin: Instant::now(),
            callback_sequences: Vec::with_capacity(8),
            callback_count: 0,
            token,
            yaw: 0.0,
            pause_started: None,
            paused_total: Duration::ZERO,
            live_mode: update_rx.is_some(),
            update_rx,
            semantic_stream: stream,
        })
    }
    /// CPU-side Instant clock is not the compositor presentation clock.
    /// Correlate ONLY via frame_sequence, not cross-clock subtraction.
    fn mark_stage(
        &mut self,
        sequence: u64,
        step: &'static str,
        measured_at: Instant,
    ) -> Result<(), String> {
        if let Some(out) = self.stage_trace.as_mut() {
            let ns = measured_at
                .saturating_duration_since(self.stage_origin)
                .as_nanos();
            serde_json::to_writer(
                &mut *out,
                &serde_json::json!({
                    "type":"cpu_stage","frame_sequence":sequence,
                    "stage":step,"time_since_host_ready_ns":ns,
                    "wl_surface_token":self.token,
                }),
            )
            .map_err(|e| e.to_string())?;
            out.write_all(b"\n").map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    fn optical_event(&mut self, value: serde_json::Value) -> Result<(), String> {
        if let Some(out) = self.optical_trace.as_mut() {
            serde_json::to_writer(&mut *out, &value).map_err(|e| e.to_string())?;
            out.write_all(b"\n").map_err(|e| e.to_string())?;
        }
        Ok(())
    }
    fn record_feedback(&mut self) -> Result<(), String> {
        for r in self.gpu.poll()? {
            if let Some((trial, bright)) = self.optical_feedback_pending.remove(&r.frame_sequence) {
                self.optical_event(serde_json::json!({
                    "type":"compositor_outcome","frame_sequence":r.frame_sequence,
                    "trial":trial,"roi_bright_when_submitted":bright,
                    "outcome":if r.discarded{"discarded"}else{"presented"},
                    "time_source":"wp_presentation_not_photons",
                }))?;
            }
            if r.frame_sequence == 1 {
                if self.first_frame_outcome.is_some() {
                    return Err("GPU05_PACER06_DUPLICATE_FIRST_FRAME_OUTCOME".into());
                }
                self.first_frame_outcome = Some(if r.discarded {
                    Outcome::Discarded
                } else {
                    Outcome::Presented
                });
                println!(
                    "GPU05_PACER06_FIRST_OUTCOME outcome={} via=wp_presentation",
                    if r.discarded {
                        "discarded"
                    } else {
                        "presented"
                    }
                );
            }
            let mut item = serde_json::json!({
                "type":"feedback","feedback_id":r.feedback_id,
                "frame_sequence":r.frame_sequence,"wl_surface_token":self.token,
                "outcome":if r.discarded {"discarded"}else{"presented"}
            });
            if r.discarded {
                self.discarded_frames += 1;
            } else if let Some(ns) = r.timestamp_ns {
                if self.first_compositor_present.is_none() {
                    self.first_compositor_present = Some((r.frame_sequence, ns));
                }
                if let Some(refresh) = r.refresh_ns {
                    if refresh > 0 {
                        self.last_feedback_refresh_ns = Some(refresh);
                    }
                }
                self.presented_frames += 1;
                let sec = ns / 1_000_000_000u128;
                item["tv_sec_hi"] = serde_json::json!((sec >> 32) as u32);
                item["tv_sec_lo"] = serde_json::json!(sec as u32);
                item["tv_nsec"] = serde_json::json!((ns % 1_000_000_000u128) as u32);
                item["refresh_ns"] = serde_json::json!(r.refresh_ns.unwrap_or(0));
            }
            self.mark_stage(
                r.frame_sequence,
                if r.discarded {
                    "compositor_discard_observed"
                } else {
                    "compositor_present_observed"
                },
                Instant::now(),
            )?;
            if let Some(out) = self.trace.as_mut() {
                serde_json::to_writer(&mut *out, &item).map_err(|e| e.to_string())?;
                out.write_all(b"\n").map_err(|e| e.to_string())?;
            }
        }
        for cb in self.gpu.frame_callbacks()? {
            if self.callback_sequences.len() >= 8 {
                return Err("GPU05_CALLBACK_APP_QUEUE_FULL".into());
            }
            self.callback_count += 1;
            self.mark_stage(cb.frame_sequence, "frame_callback_observed", Instant::now())?;
            if let Some(out) = self.callback_trace.as_mut() {
                serde_json::to_writer(
                    &mut *out,
                    &serde_json::json!({
                        "type":"wl_surface.frame.done",
                        "frame_sequence":cb.frame_sequence,
                        "callback_data_ms":cb.compositor_callback_ms,
                        "wl_surface_token":self.token,
                    }),
                )
                .map_err(|e| e.to_string())?;
                out.write_all(b"\n").map_err(|e| e.to_string())?;
            }
            self.callback_sequences.push(cb.frame_sequence);
        }
        for button in self.gpu.optical_buttons()? {
            let now = self.started.elapsed();
            let result = if let Some(probe) = self.optical_probe.as_mut() {
                probe.on_button(button.pressed, now)?
            } else {
                None
            };
            self.optical_event(serde_json::json!({
                "type":"wayland_button","button":"BTN_LEFT",
                "pressed":button.pressed,"compositor_event_time_ms":button.compositor_time_ms,
                "provider_id":"wayland-gpu05-owned-surface",
                "pointer_id":"primary","provider_sequence":button.provider_sequence,
                "surface_x":button.surface_x,"surface_y":button.surface_y,
                "coordinates_space":"surface_logical_pixels",
                "wl_surface_token":self.token,
                "host_receive_elapsed_ns":now.as_nanos(),
                "timing_class":"unattested_wayland_software_input",
            }))?;
            if let Some(Transition::Bright(trial)) = result {
                self.optical_event(serde_json::json!({
                    "type":"probe_bright_requested","trial":trial,
                    "host_elapsed_ns":now.as_nanos(),
                    "authority":"renderer_local_witness_only",
                }))?;
            }
        }
        Ok(())
    }
    fn render(&mut self) -> Result<(), String> {
        self.record_feedback()?;
        let (width, height) = self.gpu.size();
        if width == 0 || height == 0 || width > 4096 || height > 4096 {
            return Err("GPU05_XDG_SURFACE_SIZE_OUT_OF_BOUNDS".into());
        }
        if width != self.config.width || height != self.config.height {
            self.config.width = width;
            self.config.height = height;
            self.gpu
                .wgpu_surface()
                .configure(&self.device, &self.config);
            println!("GPU05_XDG_SURFACE_RECONFIGURED width={width} height={height}");
        }
        if let (Some(rx), Some(stream)) = (&self.update_rx, &mut self.semantic_stream) {
            for _ in 0..16 {
                match rx.try_recv() {
                    Ok(message) => match stream.apply_live_bytes(&message) {
                        Ok(sequence) => println!("GPU04_LIVE_DELTA_ACCEPTED sequence={sequence}"),
                        Err(reason) => eprintln!("GPU04_LIVE_DELTA_REJECTED reason={reason}"),
                    },
                    Err(std::sync::mpsc::TryRecvError::Empty) => break,
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        eprintln!("GPU04_DELTA_READER_DISCONNECTED");
                        break;
                    }
                }
            }
        }
        let stage_seq = self.next_frame_sequence;
        self.mark_stage(stage_seq, "before_acquire", Instant::now())?;
        let frame = match self.gpu.wgpu_surface().get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(v)
            | wgpu::CurrentSurfaceTexture::Suboptimal(v) => v,
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                self.gpu
                    .wgpu_surface()
                    .configure(&self.device, &self.config);
                self.wsi_timeout_streak = 0;
                println!("GPU05_WSI_RECONFIGURE_LOST_OR_OUTDATED");
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Timeout => {
                self.wsi_timeout_streak += 1;
                self.wsi_timeout_total += 1;
                eprintln!(
                    "GPU05_WSI_TIMEOUT streak={} total={} NOT_SUBMITTED",
                    self.wsi_timeout_streak, self.wsi_timeout_total
                );
                if self.wsi_timeout_streak >= 3 {
                    return Err("GPU05_WSI_TIMEOUT_REPEATED_ABORT".into());
                }
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Occluded => {
                self.wsi_occluded_total += 1;
                if self.wsi_occluded_total == 1 {
                    println!("GPU05_SURFACE_OCCLUDED_NOT_SUBMITTED");
                }
                return Ok(());
            }
            other => return Err(format!("SURFACE_UNAVAILABLE {other:?}")),
        };
        self.wsi_timeout_streak = 0;
        self.mark_stage(stage_seq, "after_acquire", Instant::now())?;
        let observed_at = self.pause_started.unwrap_or_else(Instant::now);
        let now = self.started.elapsed();
        if let Some(probe) = self.optical_probe.as_mut() {
            let transition = probe.tick(now);
            if let Some(Transition::Dark(trial)) = transition {
                self.optical_event(serde_json::json!({
                    "type":"probe_dark_reset","trial":trial,
                    "host_elapsed_ns":now.as_nanos(),
                }))?;
            }
        }
        let optic_trial = self
            .optical_probe
            .as_ref()
            .and_then(OpticalProbe::active_trial);
        let optic_bright = self
            .optical_probe
            .as_ref()
            .is_some_and(OpticalProbe::is_bright);
        if let Some(buffer) = self.optical_uniform.as_ref() {
            self.queue.write_buffer(
                buffer,
                0,
                bytemuck::cast_slice(&[if optic_bright { 1.0f32 } else { 0.0 }, 0.0, 0.0, 0.0]),
            );
        }
        let seconds = observed_at
            .duration_since(self.started)
            .saturating_sub(self.paused_total)
            .as_secs_f32();
        self.queue.write_buffer(
            &self.params,
            0,
            bytemuck::cast_slice(&[
                seconds,
                self.yaw,
                self.semantic_stream.as_ref().map_or_else(
                    || (seconds * 0.75).sin() * 0.11,
                    |stream| {
                        if self.live_mode {
                            stream.latest_offset()
                        } else {
                            stream.cavity_offset_at(seconds)
                        }
                    },
                ),
                if self.semantic_stream.is_some() {
                    0.0
                } else {
                    seconds
                },
            ]),
        );
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("gpu02 live compute->present frame"),
            });
        {
            let mut c = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("gpu02 live raymarch"),
                timestamp_writes: None,
            });
            c.set_pipeline(&self.compute);
            c.set_bind_group(0, &self.compute_bind, &[]);
            c.dispatch_workgroups(W.div_ceil(8), H.div_ceil(8), 1);
        }
        encoder.copy_buffer_to_texture(
            wgpu::TexelCopyBufferInfo {
                buffer: &self.rgba,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(W * 4),
                    rows_per_image: Some(H),
                },
            },
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d {
                width: W,
                height: H,
                depth_or_array_layers: 1,
            },
        );
        {
            let view = frame
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("gpu02 compositor surface"),
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
            });
            pass.set_pipeline(&self.draw);
            pass.set_bind_group(0, &self.draw_bind, &[]);
            pass.draw(0..3, 0..1);
        }
        self.mark_stage(stage_seq, "before_queue_submit", Instant::now())?;
        self.queue.submit([encoder.finish()]);
        self.mark_stage(stage_seq, "after_queue_submit", Instant::now())?;
        self.gpu.arm(self.next_frame_sequence)?;
        self.mark_stage(stage_seq, "after_feedback_arm", Instant::now())?;
        if self.optical_probe.is_some() {
            let trial = optic_trial.unwrap_or(0);
            if self
                .optical_feedback_pending
                .insert(stage_seq, (trial, optic_bright))
                .is_some()
            {
                return Err("ORACLE02_DUPLICATE_OPTICAL_FRAME_SEQUENCE".into());
            }
            if self.optical_feedback_pending.len() > 8 {
                return Err("ORACLE02_PENDING_FRAME_LIMIT_EXCEEDED".into());
            }
            self.optical_event(serde_json::json!({
                "type":"roi_frame_submitted","frame_sequence":stage_seq,
                "trial":trial,"bright":optic_bright,
                "time_source":"gpu_submission_not_photons",
            }))?;
        }
        if stage_seq == 1 {
            self.first_submit_begin_mono_ns = Some(monotonic_now_ns()?);
        }
        self.next_frame_sequence += 1;
        self.queue.present(frame);
        if stage_seq == 1 {
            self.first_submit_return_mono_ns = Some(monotonic_now_ns()?);
        }
        self.mark_stage(stage_seq, "after_queue_present", Instant::now())?;
        self.record_feedback()?;
        // Measures successful queue.present calls ONLY, never compositor display timestamps.
        let submitted = Instant::now();
        if let Some(prev) = self.previous_submit_at {
            if self.submit_intervals_ms.len() < 2048 {
                self.submit_intervals_ms
                    .push(submitted.duration_since(prev).as_secs_f64() * 1_000.0);
            }
        }
        self.previous_submit_at = Some(submitted);
        self.frames += 1;
        if self.report_at.elapsed() >= Duration::from_secs(2) {
            let dt = self.report_at.elapsed().as_secs_f64();
            println!("GPU02_PRESENT_SUBMITTED frames={} interval_s={:.3} submit_rate_hz={:.2} NOT_COMPOSITOR_PRESENTED_FPS",
                self.frames,dt,self.frames as f64/dt);
            self.submit_intervals_ms.sort_unstable_by(f64::total_cmp);
            if !self.submit_intervals_ms.is_empty() {
                let values = &self.submit_intervals_ms;
                let n = values.len();
                let p95 = values[((n - 1) * 95) / 100];
                let p99 = values[((n - 1) * 99) / 100];
                let max = values[n - 1];
                println!("GPU04_QUEUE_PRESENT_INTERARRIVAL count={n} p95_ms={p95:.3} p99_ms={p99:.3} max_ms={max:.3} NOT_WP_PRESENTATION");
                self.submit_intervals_ms.clear();
            }
            println!(
                "GPU05_FEEDBACK presented={} discarded={} pending={} frame_callbacks={} callback_pending={}",
                self.presented_frames,
                self.discarded_frames,
                self.gpu.pending(),
                self.callback_count,
                self.gpu.callback_pending()
            );
            if let Some(out) = self.trace.as_mut() {
                out.flush().map_err(|e| e.to_string())?;
            }
            self.frames = 0;
            self.report_at = Instant::now();
        }
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut seconds = 8u64;
    let mut scene = None;
    let mut output = None;
    let mut callback_output = None;
    let mut stage_output = None;
    let mut startup_clock_output: Option<String> = None;
    let mut pinned_output: Option<String> = None;
    let mut startup_policy = StartupPolicy::Immediate;
    let mut initial_feedback_mode = StartupMode::Current;
    let mut optical_trace_output: Option<String> = None;
    let mut host_stall_output: Option<String> = None;
    let mut args = std::env::args().skip(1);
    while let Some(key) = args.next() {
        match key.as_str() {
            "--seconds" => {
                seconds = args.next().ok_or("--seconds requires integer")?.parse()?;
                if !(2..=180).contains(&seconds) {
                    return Err("GPU05_DURATION_OUT_OF_RANGE".into());
                }
            }
            "--scene-stream" => {
                scene = Some(SceneStream::load(Path::new(
                    &args.next().ok_or("--scene-stream requires path")?,
                ))?);
            }
            "--feedback-jsonl" => {
                let value = args
                    .next()
                    .ok_or("--feedback-jsonl requires absolute path")?;
                if !Path::new(&value).is_absolute() {
                    return Err("GPU05_TRACE_PATH_NOT_ABSOLUTE".into());
                }
                output = Some(value);
            }
            "--callback-jsonl" => {
                let value = args
                    .next()
                    .ok_or("--callback-jsonl requires absolute path")?;
                if !Path::new(&value).is_absolute() {
                    return Err("GPU05_CALLBACK_TRACE_PATH_NOT_ABSOLUTE".into());
                }
                callback_output = Some(value);
            }
            "--first-feedback-gate" => {
                let value = args.next().ok_or("--first-feedback-gate requires value")?;
                initial_feedback_mode = StartupMode::parse(&value)?;
            }
            "--startup-policy" => {
                let value = args.next().ok_or("--startup-policy requires value")?;
                startup_policy = StartupPolicy::parse(&value)?;
            }
            "--pin-output" => {
                let name = args.next().ok_or("--pin-output requires a monitor name")?;
                if !matches!(name.as_str(), "HDMI-A-1" | "DP-2") {
                    return Err("GPU05_OUTPUT_NOT_IN_REFERENCE_ALLOWLIST".into());
                }
                pinned_output = Some(name);
            }
            "--startup-clock-json" => {
                let path = args
                    .next()
                    .ok_or("--startup-clock-json requires absolute path")?;
                if !Path::new(&path).is_absolute() {
                    return Err("GPU05_STARTUP_CLOCK_PATH_NOT_ABSOLUTE".into());
                }
                startup_clock_output = Some(path);
            }
            "--optical-probe-jsonl" => {
                let value = args
                    .next()
                    .ok_or("--optical-probe-jsonl requires absolute path")?;
                if !Path::new(&value).is_absolute() {
                    return Err("ORACLE02_TRACE_PATH_NOT_ABSOLUTE".into());
                }
                optical_trace_output = Some(value);
            }
            "--host-stall-jsonl" => {
                let path = args
                    .next()
                    .ok_or("--host-stall-jsonl requires absolute path")?;
                if !Path::new(&path).is_absolute() {
                    return Err("ORACLE04_STALL_TRACE_PATH_NOT_ABSOLUTE".into());
                }
                host_stall_output = Some(path);
            }
            "--stage-jsonl" => {
                let value = args.next().ok_or("--stage-jsonl requires path")?;
                if !Path::new(&value).is_absolute() {
                    return Err("GPU05_STAGE_TRACE_PATH_NOT_ABSOLUTE".into());
                }
                stage_output = Some(value);
            }
            _ => return Err("GPU05_UNKNOWN_ARGUMENT".into()),
        }
    }
    let trace = match output {
        Some(path) => Some(BufWriter::new(
            OpenOptions::new().write(true).create_new(true).open(path)?,
        )),
        None => None,
    };
    let callback_trace = match callback_output {
        Some(path) => Some(BufWriter::new(
            OpenOptions::new().write(true).create_new(true).open(path)?,
        )),
        None => None,
    };
    let stage_trace = match stage_output {
        Some(path) => Some(BufWriter::new(
            OpenOptions::new().write(true).create_new(true).open(path)?,
        )),
        None => None,
    };
    let mut host_stall_trace: Option<BufWriter<File>> = match host_stall_output {
        Some(path) => Some(BufWriter::new(
            OpenOptions::new().write(true).create_new(true).open(path)?,
        )),
        None => None,
    };
    if let Some(out) = host_stall_trace.as_mut() {
        serde_json::to_writer(
            &mut *out,
            &serde_json::json!({
                "type":"header","schema":"blobin.gpu05.host-stall.v1",
                "monotonic_domains":["host_process_instant","thread_cpu_clock"],
                "presentation_clock":"separate_wp_presentation",
                "counter_scope":"GPU05_OPTIN_FORENSIC_ONLY",
                "max_presentation_gap_threshold_ns":33_400_000,
                "causal_attribution":"NOT_ESTABLISHED",
            }),
        )?;
        out.write_all(b"\n")?;
    }
    let optical_trace = match optical_trace_output {
        Some(path) => Some(BufWriter::new(
            OpenOptions::new().write(true).create_new(true).open(path)?,
        )),
        None => None,
    };
    let mut live = pollster::block_on(Live::create(
        scene,
        None,
        trace,
        callback_trace,
        stage_trace,
        pinned_output.as_deref(),
        optical_trace,
    ))?;
    let app_ready_mono_ns = monotonic_now_ns()?;
    println!("GPU05_PACER04_STAGES_CPU_CLOCK_INSTANT not_wayland_clock=true");
    // Deliberate one-variable A/B after scene import, device setup and
    // swapchain configuration, but before the first surface frame.
    // This is NOT an unrecorded warmup frame nor proof of GPU idle.
    let settle = startup_policy.settle_duration();
    println!(
        "GPU05_PACER05_STARTUP variant={} settle_ms={} window_ready=true",
        startup_policy.name(),
        settle.as_millis()
    );
    if !settle.is_zero() {
        std::thread::sleep(settle);
    }
    let start = Instant::now();
    let mut pacing = FrameCallbackPacing::new();
    let mut first_barrier = FirstFrameBarrier::new(initial_feedback_mode);
    let mut first_outcome_propagated = false;
    let mut host_stall_probe = HostStallProbe::new();
    let mut max_wayland_poll_ns = 0_u64;
    println!(
        "GPU05_PACER06_GATE mode={} max_first_wait_ms=250",
        first_barrier.mode().name()
    );
    let mut abort_reason: Option<&'static str> = None;
    println!(
        "GPU05_PACER03_ACTIVE frame_credit=wl_surface.frame max_wp_inflight=2 watchdog_ms=250"
    );
    while start.elapsed() < Duration::from_secs(seconds) && !live.gpu.closed() {
        let gap = if host_stall_trace.is_some() {
            let instant_ns = u64::try_from(start.elapsed().as_nanos())?;
            let cpu_ns = thread_cpu_time_ns()?;
            host_stall_probe.sample(instant_ns, cpu_ns)?
        } else {
            None
        };
        let before_feedback = live.presented_frames + live.discarded_frames;
        let poll_start = Instant::now();
        let poll_cpu_begin = if host_stall_trace.is_some() {
            Some(thread_cpu_time_ns()?)
        } else {
            None
        };
        live.record_feedback()?;
        let poll_ns = u64::try_from(poll_start.elapsed().as_nanos())?;
        let poll_cpu_delta = if let Some(begin) = poll_cpu_begin {
            Some(thread_cpu_time_ns()?.saturating_sub(begin))
        } else {
            None
        };
        max_wayland_poll_ns = max_wayland_poll_ns.max(poll_ns);
        if let Some(out) = host_stall_trace.as_mut() {
            if gap.is_some_and(|g| g.exceeds_33_4ms) || poll_ns > 25_000_000 {
                serde_json::to_writer(
                    &mut *out,
                    &serde_json::json!({
                        "type":"stall_outlier",
                        "next_frame_sequence":live.next_frame_sequence,
                        "loop_iterations":host_stall_probe.iterations,
                        "loop_gap_wall_ns":gap.map(|g|g.wall_gap_ns),
                        "loop_gap_thread_cpu_ns":gap.map(|g|g.thread_cpu_gap_ns),
                        "loop_gap_wall_minus_cpu_ns":gap.map(|g|g.wall_minus_cpu_ns),
                        "wall_gap_over_33_4ms":gap.is_some_and(|g|g.exceeds_33_4ms),
                        "wayland_poll_wall_ns":poll_ns,
                        "wayland_poll_thread_cpu_ns":poll_cpu_delta,
                        "wp_feedback_pending":live.gpu.pending(),
                        "frame_callback_pending":live.gpu.callback_pending(),
                        "submitted_frames":live.next_frame_sequence-1,
                        "note":"thread_wall_minus_cpu_is_not_proof_of_preemption",
                    }),
                )?;
                out.write_all(b"\n")?;
            }
        }
        let now = start.elapsed();
        if !first_outcome_propagated {
            if let Some(outcome) = live.first_frame_outcome {
                if let Err(reason) = first_barrier.first_feedback(1, outcome) {
                    abort_reason = Some(reason);
                    break;
                }
                first_outcome_propagated = true;
            }
        }
        for seq in std::mem::take(&mut live.callback_sequences) {
            if let Err(reason) = pacing.on_frame_callback(seq, now) {
                abort_reason = Some(reason);
                break;
            }
        }
        if abort_reason.is_some() {
            break;
        }
        if live.presented_frames + live.discarded_frames > before_feedback {
            pacing.on_presentation_feedback(now);
        }
        match pacing.should_submit(live.gpu.pending(), now) {
            Ok(true) => {}
            Ok(false) => {
                std::thread::sleep(Duration::from_millis(1));
                continue;
            }
            Err(reason) => {
                abort_reason = Some(reason);
                break;
            }
        }
        match first_barrier.permit(live.next_frame_sequence, now) {
            Ok(true) => {}
            Ok(false) => {
                std::thread::sleep(Duration::from_millis(1));
                continue;
            }
            Err(reason) => {
                abort_reason = Some(reason);
                break;
            }
        }
        let before_seq = live.next_frame_sequence;
        let before_feedback = live.presented_frames + live.discarded_frames;
        let render_before = Instant::now();
        if let Err(e) = live.render() {
            eprintln!("GPU05_PACER03_FRAME_ABORT reason={e}");
            abort_reason = Some("GPU05_PACER03_FRAME_ERROR");
            break;
        }
        let render_ns = u64::try_from(render_before.elapsed().as_nanos())?;
        if let Some(out) = host_stall_trace.as_mut() {
            if render_ns > 25_000_000 {
                serde_json::to_writer(
                    &mut *out,
                    &serde_json::json!({
                        "type":"render_outlier",
                        "frame_sequence":before_seq,
                        "render_wall_ns":render_ns,
                        "note":"GPU05_client_render_scope_including_wsi_not_photons"
                    }),
                )?;
                out.write_all(b"\n")?;
            }
        }
        let after_render = start.elapsed();
        if live.next_frame_sequence > before_seq {
            if let Err(reason) = pacing.submitted(before_seq, after_render) {
                abort_reason = Some(reason);
                break;
            }
            if let Err(reason) = first_barrier.submitted(before_seq, after_render) {
                abort_reason = Some(reason);
                break;
            }
        }
        if !first_outcome_propagated {
            if let Some(outcome) = live.first_frame_outcome {
                if let Err(reason) = first_barrier.first_feedback(1, outcome) {
                    abort_reason = Some(reason);
                    break;
                }
                first_outcome_propagated = true;
            }
        }
        for seq in std::mem::take(&mut live.callback_sequences) {
            if let Err(reason) = pacing.on_frame_callback(seq, after_render) {
                abort_reason = Some(reason);
                break;
            }
        }
        if abort_reason.is_some() {
            break;
        }
        if live.presented_frames + live.discarded_frames > before_feedback {
            pacing.on_presentation_feedback(after_render);
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    // Settle the final compositor feedback before closing the owned surface.
    // No new frames are produced during this bounded shutdown drain.
    for _ in 0..200 {
        live.record_feedback()?;
        // A terminal frame callback can arrive during the shutdown drain.
        // Count its exact sequence without granting another GPU submission.
        for seq in std::mem::take(&mut live.callback_sequences) {
            pacing.on_frame_callback(seq, start.elapsed())?;
        }
        if !first_outcome_propagated {
            if let Some(outcome) = live.first_frame_outcome {
                first_barrier.first_feedback(1, outcome)?;
                first_outcome_propagated = true;
            }
        }
        if live.gpu.pending() == 0
            && live.gpu.callback_pending() == 0
            && pacing.callback_pending() == 0
        {
            break;
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    if let Some(out) = host_stall_trace.as_mut() {
        serde_json::to_writer(
            &mut *out,
            &serde_json::json!({
                "type":"summary",
                "iterations":host_stall_probe.iterations,
                "loop_gap_outliers":host_stall_probe.outliers,
                "max_loop_gap_ns":host_stall_probe.max_loop_gap_ns,
                "max_thread_cpu_gap_ns":host_stall_probe.max_thread_cpu_gap_ns,
                "max_wayland_poll_ns":max_wayland_poll_ns,
                "cp03_pass":false,
            }),
        )?;
        out.write_all(b"\n")?;
        out.flush()?;
    }
    if let Some(out) = live.trace.as_mut() {
        out.flush()?;
    }
    if let Some(out) = live.callback_trace.as_mut() {
        out.flush()?;
    }
    if let Some(out) = live.stage_trace.as_mut() {
        out.flush()?;
    }
    if let Some(out) = live.optical_trace.as_mut() {
        out.flush()?;
    }
    if live.optical_probe.is_some() {
        println!(
            "ORACLE02_EXIT pending_optical_frames={} optical_physical_capture=NOT_RUN",
            live.optical_feedback_pending.len()
        );
        if !live.optical_feedback_pending.is_empty() {
            return Err("ORACLE02_UNRESOLVED_OPTICAL_FRAME_OUTCOMES".into());
        }
    }
    if let Some(path) = startup_clock_output {
        let clock = live.gpu.clock().ok_or("GPU05_MISSING_WP_CLOCK_ID")?;
        let (presented_seq, presented_mono) = live
            .first_compositor_present
            .ok_or("GPU05_MISSING_FIRST_COMPOSITOR_PRESENTATION")?;
        let samples = StartupAnchors {
            app_ready_ns: app_ready_mono_ns,
            first_submit_begin_ns: live
                .first_submit_begin_mono_ns
                .ok_or("GPU05_MISSING_FIRST_SUBMIT_BEGIN")?,
            first_submit_return_ns: live
                .first_submit_return_mono_ns
                .ok_or("GPU05_MISSING_FIRST_SUBMIT_RETURN")?,
            first_compositor_present_ns: presented_mono,
        };
        let latency = calculate(clock, samples)?;
        let payload = serde_json::json!({
            "schema":"blobin.gpu05.startup-clock.v1",
            "clock_id":clock,
            "matched_clock":"CLOCK_MONOTONIC",
            "source":"same-surface-wp_presentation",
            "wl_surface_token":live.token,
            "first_compositor_present_sequence":presented_seq,
            "first_frame_outcome":match live.first_frame_outcome {
                Some(Outcome::Presented)=>"presented",
                Some(Outcome::Discarded)=>"discarded",
                None=>"unresolved",
            },
            "app_ready_mono_ns":samples.app_ready_ns.to_string(),
            "first_submit_begin_mono_ns":samples.first_submit_begin_ns.to_string(),
            "first_submit_return_mono_ns":samples.first_submit_return_ns.to_string(),
            "first_compositor_present_mono_ns":samples.first_compositor_present_ns.to_string(),
            "ready_to_present_ms":latency.ready_to_present_ns as f64/1e6,
            "submit_begin_to_present_ms":latency.submit_begin_to_present_ns as f64/1e6,
            "submit_return_to_present_ms":latency.submit_return_to_present_ns
                .map(|v|v as f64/1e6),
            "is_input_to_photon_latency":false,
            "qualifies_cp03":false,
        });
        let mut out = BufWriter::new(OpenOptions::new().write(true).create_new(true).open(path)?);
        serde_json::to_writer_pretty(&mut out, &payload)?;
        out.write_all(b"\n")?;
        out.flush()?;
        println!("GPU05_PACER07_FIRST_PRESENT_LATENCY_MS={:.3} sequence={} wp_clock_id={} NOT_INPUT_TO_PHOTON",
            latency.ready_to_present_ns as f64/1e6,presented_seq,clock);
    }
    println!("GPU05_PACER06_EXIT mode={} first_outcome={:?} startup_wait_iterations={} seconds={:.3} submitted={} compositor_presented={} discarded={} pending={} callbacks={} callback_pending={} max_inflight={} wsi_timeouts={} wsi_occluded={} watchdog_trips={} aborted={} NOT_CP03",
        first_barrier.mode().name(),first_barrier.outcome(),
        first_barrier.wait_iterations(),start.elapsed().as_secs_f64(),live.next_frame_sequence-1,
        live.presented_frames,live.discarded_frames,live.gpu.pending(),
        pacing.callbacks(),pacing.callback_pending(),
        pacing.max_inflight(),live.wsi_timeout_total,live.wsi_occluded_total,
        pacing.watchdog_trips(),abort_reason.is_some());
    if let Some(reason) = abort_reason {
        return Err(reason.into());
    }
    if live.gpu.pending() != 0 {
        return Err("GPU05_UNRESOLVED_FEEDBACK_ON_SHUTDOWN".into());
    }
    if live.gpu.callback_pending() != 0 || pacing.callback_pending() != 0 {
        return Err("GPU05_UNRESOLVED_FRAME_CALLBACK_ON_SHUTDOWN".into());
    }
    if pacing.callbacks() != live.next_frame_sequence - 1 {
        return Err("GPU05_FRAME_CALLBACK_IDENTITY_PARTITION_MISMATCH".into());
    }
    Ok(())
}
