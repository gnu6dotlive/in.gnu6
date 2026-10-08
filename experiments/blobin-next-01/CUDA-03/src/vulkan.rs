//! Persistent wgpu CSG compute session; host-synchronized wall timings only.
use blobin_cuda02_provider::{Request, Sample};
use std::{sync::mpsc, time::Duration};
use wgpu::util::DeviceExt;

pub struct Resident {
    dims: (u32, u32),
    count: usize,
    device: wgpu::Device,
    queue: wgpu::Queue,
    primitive: wgpu::Buffer,
    output: wgpu::Buffer,
    pipeline: wgpu::ComputePipeline,
    bind: wgpu::BindGroup,
}
fn primitives_bytes(req: &Request) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(req.primitives().len() * 16);
    for p in req.primitives() {
        for v in [p.x, p.y, p.radius, p.op as f32] {
            bytes.extend_from_slice(&v.to_le_bytes());
        }
    }
    bytes
}
impl Resident {
    pub fn new(req: &Request) -> Result<Self, String> {
        req.validate()?;
        let mut desc = wgpu::InstanceDescriptor::new_without_display_handle();
        desc.backends = wgpu::Backends::VULKAN;
        let instance = wgpu::Instance::new(desc);
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: None,
            ..Default::default()
        }))
        .map_err(|e| format!("CUDA03 VULKAN ADAPTER {e}"))?;
        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
                .map_err(|e| format!("CUDA03 VULKAN DEVICE {e}"))?;
        let (w, h) = req.dimensions();
        let mut uniform = Vec::with_capacity(16);
        for n in [w, h, req.primitives().len() as u32, 0] {
            uniform.extend_from_slice(&n.to_le_bytes());
        }
        let conf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("cuda03-conf"),
            contents: &uniform,
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let primitive = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("cuda03-prims"),
            contents: &primitives_bytes(req),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });
        let output = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("cuda03-output"),
            size: w as u64 * h as u64 * 16,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("cuda03-geometry"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("../../CUDA-02/shaders/geometry.wgsl").into(),
            ),
        });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("cuda03-persistent-csg"),
            layout: None,
            module: &shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });
        let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("cuda03-resident-bind"),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: conf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: primitive.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: output.as_entire_binding(),
                },
            ],
        });
        Ok(Self {
            dims: (w, h),
            count: req.primitives().len(),
            device,
            queue,
            primitive,
            output,
            pipeline,
            bind,
        })
    }
    pub fn resident_bytes(&self) -> u64 {
        16 + self.count as u64 * 16 + self.dims.0 as u64 * self.dims.1 as u64 * 16
    }
    pub fn update(&mut self, req: &Request) -> Result<(), String> {
        req.validate()?;
        if req.dimensions() != self.dims || req.primitives().len() != self.count {
            return Err("CUDA03 VULKAN SHAPE CHANGED".into());
        }
        self.queue
            .write_buffer(&self.primitive, 0, &primitives_bytes(req));
        Ok(())
    }
    pub fn launch(&self) -> Result<(), String> {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("cuda03-dispatch"),
            });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("cuda03-compute-only"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.bind, &[]);
            pass.dispatch_workgroups(self.dims.0.div_ceil(8), self.dims.1.div_ceil(8), 1);
        }
        let index = self.queue.submit(Some(encoder.finish()));
        self.device
            .poll(wgpu::PollType::Wait {
                submission_index: Some(index),
                timeout: Some(Duration::from_secs(20)),
            })
            .map_err(|e| format!("CUDA03 VULKAN WAIT {e}"))?;
        Ok(())
    }
    pub fn readback(&self) -> Result<Vec<Sample>, String> {
        let size = self.dims.0 as u64 * self.dims.1 as u64 * 16;
        let staging = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("cuda03-staging"),
            size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("cuda03-readback"),
            });
        encoder.copy_buffer_to_buffer(&self.output, 0, &staging, 0, size);
        self.queue.submit(Some(encoder.finish()));
        let slice = staging.slice(..);
        let (tx, rx) = mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = tx.send(result);
        });
        self.device
            .poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: Some(Duration::from_secs(20)),
            })
            .map_err(|e| format!("CUDA03 VULKAN MAP WAIT {e}"))?;
        rx.recv_timeout(Duration::from_secs(20))
            .map_err(|e| format!("CUDA03 VULKAN MAP TIMEOUT {e}"))?
            .map_err(|e| format!("CUDA03 VULKAN MAP ERROR {e}"))?;
        let data = slice.get_mapped_range().map_err(|e| e.to_string())?;
        if data.len() != size as usize {
            return Err("CUDA03 VULKAN LENGTH".into());
        }
        let mut samples = Vec::with_capacity(self.dims.0 as usize * self.dims.1 as usize);
        for chunk in data.chunks_exact(16) {
            let read =
                |i: usize| f32::from_le_bytes(chunk[i..i + 4].try_into().expect("four bytes"));
            samples.push(Sample {
                distance: read(0),
                nx: read(4),
                ny: read(8),
                coverage: read(12),
            });
        }
        drop(data);
        staging.unmap();
        Ok(samples)
    }
}
