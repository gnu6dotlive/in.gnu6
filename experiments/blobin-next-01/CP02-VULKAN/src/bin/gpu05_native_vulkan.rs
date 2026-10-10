//! GPU-05 native owned Wayland surface, actual signed tetra/SDF shader, Vulkan WSI.
#[path = "../frame_pacing.rs"]
mod frame_pacing;
#[path = "../owned_wayland_presentation.rs"]
mod owned_wayland_presentation;
#[path = "../scene_bridge.rs"]
mod scene_bridge;
use blobin_cp01_hybrid::gltf::import_glb;
use frame_pacing::FramePacing;
use owned_wayland_presentation::OwnedGpuWindow;
use scene_bridge::SceneStream;
use std::{
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
    trace: Option<BufWriter<File>>,
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
    ) -> Result<Self, Box<dyn Error>> {
        let mut desc = wgpu::InstanceDescriptor::new_without_display_handle();
        desc.backends = wgpu::Backends::VULKAN;
        let instance = wgpu::Instance::new(desc);
        let gpu = OwnedGpuWindow::open(&instance)?;
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
        let draw_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("gpu02 draw bindings"),
            entries: &[
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
            ],
        });
        let draw_bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("gpu02 fullscreen texture"),
            layout: &draw_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texview),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });
        let draw_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("gpu02 graphics presenter"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../present.wgsl").into()),
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
            trace,
            token,
            yaw: 0.0,
            pause_started: None,
            paused_total: Duration::ZERO,
            live_mode: update_rx.is_some(),
            update_rx,
            semantic_stream: stream,
        })
    }
    fn record_feedback(&mut self) -> Result<(), String> {
        for r in self.gpu.poll()? {
            let mut item = serde_json::json!({
                "type":"feedback","feedback_id":r.feedback_id,
                "frame_sequence":r.frame_sequence,"wl_surface_token":self.token,
                "outcome":if r.discarded {"discarded"}else{"presented"}
            });
            if r.discarded {
                self.discarded_frames += 1;
            } else if let Some(ns) = r.timestamp_ns {
                self.presented_frames += 1;
                let sec = ns / 1_000_000_000u128;
                item["tv_sec_hi"] = serde_json::json!((sec >> 32) as u32);
                item["tv_sec_lo"] = serde_json::json!(sec as u32);
                item["tv_nsec"] = serde_json::json!((ns % 1_000_000_000u128) as u32);
                item["refresh_ns"] = serde_json::json!(r.refresh_ns.unwrap_or(0));
            }
            if let Some(out) = self.trace.as_mut() {
                serde_json::to_writer(&mut *out, &item).map_err(|e| e.to_string())?;
                out.write_all(b"\n").map_err(|e| e.to_string())?;
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
        let observed_at = self.pause_started.unwrap_or_else(Instant::now);
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
        self.queue.submit([encoder.finish()]);
        self.gpu.arm(self.next_frame_sequence)?;
        self.next_frame_sequence += 1;
        self.queue.present(frame);
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
                "GPU05_FEEDBACK presented={} discarded={} pending={}",
                self.presented_frames,
                self.discarded_frames,
                self.gpu.pending()
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
            _ => return Err("GPU05_UNKNOWN_ARGUMENT".into()),
        }
    }
    let trace = match output {
        Some(path) => Some(BufWriter::new(
            OpenOptions::new().write(true).create_new(true).open(path)?,
        )),
        None => None,
    };
    let mut live = pollster::block_on(Live::create(scene, None, trace))?;
    let start = Instant::now();
    let mut pacing = FramePacing::new();
    let mut abort_reason: Option<&'static str> = None;
    while start.elapsed() < Duration::from_secs(seconds) && !live.gpu.closed() {
        live.record_feedback()?;
        match pacing.allow_submit(live.gpu.pending(), start.elapsed()) {
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
        if let Err(e) = live.render() {
            eprintln!("GPU05_FRAME_LOOP_ABORT reason={e}");
            abort_reason = Some("GPU05_FRAME_LOOP_ERROR");
            break;
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    // Settle the final compositor feedback before closing the owned surface.
    // No new frames are produced during this bounded shutdown drain.
    for _ in 0..200 {
        live.record_feedback()?;
        if live.gpu.pending() == 0 {
            break;
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    if let Some(out) = live.trace.as_mut() {
        out.flush()?;
    }
    println!("GPU05_EXIT seconds={:.3} submitted={} compositor_presented={} discarded={} pending={} max_inflight={} wsi_timeouts={} wsi_occluded={} aborted={} NOT_CP03",
        start.elapsed().as_secs_f64(),live.next_frame_sequence-1,
        live.presented_frames,live.discarded_frames,live.gpu.pending(),
        pacing.max_inflight(),live.wsi_timeout_total,live.wsi_occluded_total,
        abort_reason.is_some());
    if let Some(reason) = abort_reason {
        return Err(reason.into());
    }
    if live.gpu.pending() != 0 {
        return Err("GPU05_UNRESOLVED_FEEDBACK_ON_SHUTDOWN".into());
    }
    Ok(())
}
