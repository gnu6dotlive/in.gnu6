//! wgpu/Vulkan compute implementation for the same immutable CSG request.
use crate::{Request, Sample};
use std::{sync::mpsc, time::Duration};
use wgpu::util::DeviceExt;
pub fn evaluate(req: &Request) -> Result<Vec<Sample>, String> {
    req.validate()?;
    let mut description = wgpu::InstanceDescriptor::new_without_display_handle();
    description.backends = wgpu::Backends::VULKAN;
    let instance = wgpu::Instance::new(description);
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: None,
        ..Default::default()
    }))
    .map_err(|e| format!("CUDA02 VULKAN ADAPTER UNAVAILABLE {e}"))?;
    let (device, queue) =
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
            .map_err(|e| format!("CUDA02 VULKAN DEVICE UNAVAILABLE {e}"))?;
    let mut uniform = Vec::with_capacity(16);
    for value in [req.width, req.height, req.primitives.len() as u32, 0] {
        uniform.extend_from_slice(&value.to_le_bytes());
    }
    let mut geometry = Vec::with_capacity(req.primitives.len() * 16);
    for p in &req.primitives {
        for value in [p.x, p.y, p.radius, p.op as f32] {
            geometry.extend_from_slice(&value.to_le_bytes());
        }
    }
    let conf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("cuda02-csg-uniform"),
        contents: &uniform,
        usage: wgpu::BufferUsages::UNIFORM,
    });
    let primitives = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("cuda02-csg-primitives"),
        contents: &geometry,
        usage: wgpu::BufferUsages::STORAGE,
    });
    let bytes = req.width as u64 * req.height as u64 * 16;
    let output = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("cuda02-sample-storage"),
        size: bytes,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("cuda02-sample-readback"),
        size: bytes,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("cuda02-unified-csg"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/geometry.wgsl").into()),
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("cuda02-csg-compute"),
        layout: None,
        module: &shader,
        entry_point: Some("main"),
        compilation_options: Default::default(),
        cache: None,
    });
    let layout = pipeline.get_bind_group_layout(0);
    let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("cuda02-csg-bind"),
        layout: &layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: conf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: primitives.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: output.as_entire_binding(),
            },
        ],
    });
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("cuda02-one-shot-csg"),
    });
    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("cuda02-csg-shader"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bind, &[]);
        pass.dispatch_workgroups(req.width.div_ceil(8), req.height.div_ceil(8), 1);
    }
    encoder.copy_buffer_to_buffer(&output, 0, &readback, 0, bytes);
    queue.submit(Some(encoder.finish()));
    let slice = readback.slice(..);
    let (tx, rx) = mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = tx.send(result);
    });
    device
        .poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: Some(Duration::from_secs(30)),
        })
        .map_err(|e| format!("CUDA02 VULKAN POLL FAILED {e}"))?;
    rx.recv_timeout(Duration::from_secs(30))
        .map_err(|e| format!("CUDA02 VULKAN MAP TIMEOUT {e}"))?
        .map_err(|e| format!("CUDA02 VULKAN MAP FAILURE {e}"))?;
    let mapped = slice.get_mapped_range().map_err(|e| e.to_string())?;
    if mapped.len() != bytes as usize {
        return Err("CUDA02 VULKAN LENGTH".into());
    }
    let mut samples = Vec::with_capacity(req.width as usize * req.height as usize);
    for chunk in mapped.chunks_exact(16) {
        let read = |start: usize| {
            f32::from_le_bytes(chunk[start..start + 4].try_into().expect("f32 bytes"))
        };
        samples.push(Sample {
            distance: read(0),
            nx: read(4),
            ny: read(8),
            coverage: read(12),
        });
    }
    drop(mapped);
    readback.unmap();
    Ok(samples)
}
