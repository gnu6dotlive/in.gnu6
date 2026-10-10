//! GPU-02: private Wayland host of the actual GPU-01 signed-mesh shader.
//! Uses Vulkan compute -> GPU-only copy -> Vulkan textured draw -> Wayland present.
//! No static PNG, CPU pixel substitution or BLOB-FRAME-v1 schema changes.
#[path = "../scene_bridge.rs"]
mod scene_bridge;
use blobin_cp01_hybrid::gltf::import_glb;
use scene_bridge::SceneStream;
use std::{
    error::Error,
    sync::Arc,
    time::{Duration, Instant},
};
use wgpu::util::DeviceExt;
use winit::{
    application::ApplicationHandler,
    event::{ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};
const W: u32 = 640;
const H: u32 = 432;

struct Live {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
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
    frames: u64,
    yaw: f32,
    paused: bool,
    pause_started: Option<Instant>,
    paused_total: Duration,
    semantic_stream: Option<SceneStream>,
}

impl Live {
    async fn create(
        window: Arc<Window>,
        stream: Option<SceneStream>,
    ) -> Result<Self, Box<dyn Error>> {
        let mut desc = wgpu::InstanceDescriptor::new_without_display_handle();
        desc.backends = wgpu::Backends::VULKAN;
        let instance = wgpu::Instance::new(desc);
        let surface = instance.create_surface(window.clone())?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
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
        let size = window.inner_size();
        let mut config = surface
            .get_default_config(&adapter, size.width.max(1), size.height.max(1))
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
        println!("GPU02_REAL_COMPUTE_TO_WAYLAND no_png_intermediary=true no_cpu_readback=true");
        Ok(Self {
            window,
            surface,
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
            frames: 0,
            yaw: 0.0,
            paused: false,
            pause_started: None,
            paused_total: Duration::ZERO,
            semantic_stream: stream,
        })
    }
    fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
        }
    }
    fn render(&mut self) -> Result<(), String> {
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(v)
            | wgpu::CurrentSurfaceTexture::Suboptimal(v) => v,
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                self.surface.configure(&self.device, &self.config);
                return Ok(());
            }
            other => return Err(format!("SURFACE_UNAVAILABLE {other:?}")),
        };
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
                    |stream| stream.cavity_offset_at(seconds),
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
        self.window.pre_present_notify();
        self.queue.present(frame);
        self.frames += 1;
        if self.report_at.elapsed() >= Duration::from_secs(2) {
            let dt = self.report_at.elapsed().as_secs_f64();
            println!("GPU02_PRESENT_SUBMITTED frames={} interval_s={:.3} submit_rate_hz={:.2} NOT_COMPOSITOR_PRESENTED_FPS",
                self.frames,dt,self.frames as f64/dt);
            self.frames = 0;
            self.report_at = Instant::now();
        }
        Ok(())
    }
}

#[derive(Default)]
struct Application {
    state: Option<Live>,
    stream: Option<SceneStream>,
    error: Option<String>,
}
impl ApplicationHandler for Application {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        if self.state.is_some() {
            return;
        }
        let attrs = Window::default_attributes()
            .with_title("blob.in | GPU-02 live Vulkan MeshSolid/SDF")
            .with_inner_size(winit::dpi::LogicalSize::new(960.0, 648.0));
        match el
            .create_window(attrs)
            .map_err(|e| e.to_string())
            .and_then(|w| {
                pollster::block_on(Live::create(Arc::new(w), self.stream.clone()))
                    .map_err(|e| e.to_string())
            }) {
            Ok(s) => {
                s.window.request_redraw();
                self.state = Some(s);
            }
            Err(e) => {
                self.error = Some(e);
                el.exit();
            }
        }
    }
    fn window_event(&mut self, el: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        let Some(s) = self.state.as_mut() else {
            return;
        };
        if s.window.id() != id {
            return;
        }
        match event {
            WindowEvent::CloseRequested => el.exit(),
            WindowEvent::Resized(size) => s.resize(size.width, size.height),
            WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed => {
                match event.physical_key {
                    PhysicalKey::Code(KeyCode::Escape | KeyCode::KeyQ) => el.exit(),
                    PhysicalKey::Code(KeyCode::ArrowLeft) => s.yaw -= 0.15,
                    PhysicalKey::Code(KeyCode::ArrowRight) => s.yaw += 0.15,
                    PhysicalKey::Code(KeyCode::Space) => {
                        if s.paused {
                            if let Some(at) = s.pause_started.take() {
                                s.paused_total += at.elapsed();
                            }
                            s.paused = false;
                        } else {
                            s.pause_started = Some(Instant::now());
                            s.paused = true;
                        }
                    }
                    _ => {}
                }
            }
            WindowEvent::RedrawRequested => {
                if let Err(e) = s.render() {
                    self.error = Some(format!("GPU02_SURFACE_FAILED {e:?}"));
                    el.exit();
                    return;
                }
                s.window.request_redraw();
            }
            _ => {}
        }
    }
}
fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args().skip(1);
    let stream = match args.next().as_deref() {
        None => None,
        Some("--scene-stream") => {
            let path = args.next().ok_or("--scene-stream requires a file path")?;
            if args.next().is_some() {
                return Err("unexpected GPU03 arguments".into());
            }
            let parsed = SceneStream::load(std::path::Path::new(&path))?;
            println!(
                "GPU03_SEMANTIC_SCENE_ACCEPTED frames={} source=in.gnu6 read_only=true",
                parsed.frame_count()
            );
            Some(parsed)
        }
        Some(_) => return Err("usage: gpu02_wayland [--scene-stream /absolute/file.json]".into()),
    };
    let loop_ = EventLoop::new()?;
    loop_.set_control_flow(ControlFlow::Wait);
    let mut app = Application {
        state: None,
        stream,
        error: None,
    };
    loop_.run_app(&mut app)?;
    match app.error {
        Some(e) => Err(e.into()),
        None => Ok(()),
    }
}
