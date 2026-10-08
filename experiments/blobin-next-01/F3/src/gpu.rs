//! Vulkan offscreen compute backend; returns both pixels and numeric comparison fields.
use crate::Params;
use blobin_f2_topology::Scene;
use std::{sync::mpsc, time::Duration};
use wgpu::util::DeviceExt;
pub struct GpuFrame {
    pub rgba: Vec<u8>,
    pub mask: Vec<u32>,
    pub height: Vec<f32>,
    pub heat: Vec<f32>,
    pub normal_z: Vec<f32>,
    pub adapter: String,
}
pub fn render(scene: Scene, params: Params) -> Result<GpuFrame, String> {
    scene.validate()?;
    params.validate()?;
    let mut opts = wgpu::InstanceDescriptor::new_without_display_handle();
    opts.backends = wgpu::Backends::VULKAN;
    let instance = wgpu::Instance::new(opts);
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: None,
        ..Default::default()
    }))
    .map_err(|e| format!("GPU ADAPTER UNAVAILABLE: {e}"))?;
    let info = adapter.get_info();
    let hardware = format!(
        "{} / {} / {:?} / {:?}",
        info.name, info.driver, info.backend, info.device_type
    );
    let (device, queue) =
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
            .map_err(|e| format!("GPU DEVICE UNAVAILABLE: {e}"))?;
    let mut uniform = Vec::with_capacity(48);
    for value in [
        scene.width(),
        scene.height(),
        scene.logical_width,
        scene.logical_height,
    ] {
        uniform.extend_from_slice(&value.to_le_bytes());
    }
    for value in [
        scene.dpr as f32,
        scene.phase,
        scene.layout.code() as f32,
        params.material.code() as f32,
    ] {
        uniform.extend_from_slice(&value.to_le_bytes());
    }
    for value in [params.heat_strength, 0.0f32, 0.0f32, 0.0f32] {
        uniform.extend_from_slice(&value.to_le_bytes());
    }
    let input = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("f3-frame-immutable"),
        contents: &uniform,
        usage: wgpu::BufferUsages::UNIFORM,
    });
    let pixels_size = scene.width() as u64 * scene.height() as u64 * 4;
    let fields_size = scene.width() as u64 * scene.height() as u64 * 16;
    let pixels = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("f3-gpu-rgba"),
        size: pixels_size,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let analytics = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("f3-gpu-analytic-fields"),
        size: fields_size,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let rgba_readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("f3-rgba-readback"),
        size: pixels_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let fields_readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("f3-fields-readback"),
        size: fields_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("f3-materials"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/f3.wgsl").into()),
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("f3-compute"),
        layout: None,
        module: &shader,
        entry_point: Some("main"),
        compilation_options: Default::default(),
        cache: None,
    });
    let layout = pipeline.get_bind_group_layout(0);
    let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("f3-input-and-outputs"),
        layout: &layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: input.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: pixels.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: analytics.as_entire_binding(),
            },
        ],
    });
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("f3-frame"),
    });
    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("f3-material-height-pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bind, &[]);
        pass.dispatch_workgroups(scene.width().div_ceil(8), scene.height().div_ceil(8), 1);
    }
    encoder.copy_buffer_to_buffer(&pixels, 0, &rgba_readback, 0, pixels_size);
    encoder.copy_buffer_to_buffer(&analytics, 0, &fields_readback, 0, fields_size);
    queue.submit(Some(encoder.finish()));
    let rgba_slice = rgba_readback.slice(..);
    let fields_slice = fields_readback.slice(..);
    let (tx0, rx0) = mpsc::channel();
    let (tx1, rx1) = mpsc::channel();
    rgba_slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = tx0.send(result);
    });
    fields_slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = tx1.send(result);
    });
    device
        .poll(wgpu::PollType::Wait {
            timeout: Some(Duration::from_secs(15)),
            submission_index: None,
        })
        .map_err(|e| format!("GPU POLL FAILED: {e}"))?;
    rx0.recv_timeout(Duration::from_secs(15))
        .map_err(|e| e.to_string())?
        .map_err(|e| format!("GPU MAP RGBA FAILED: {e}"))?;
    rx1.recv_timeout(Duration::from_secs(15))
        .map_err(|e| e.to_string())?
        .map_err(|e| format!("GPU MAP FIELDS FAILED: {e}"))?;
    let rgba = rgba_slice
        .get_mapped_range()
        .map_err(|e| e.to_string())?
        .to_vec();
    let fields = fields_slice
        .get_mapped_range()
        .map_err(|e| e.to_string())?
        .to_vec();
    rgba_readback.unmap();
    fields_readback.unmap();
    if rgba.len() != pixels_size as usize || fields.len() != fields_size as usize {
        return Err("GPU readback length mismatch".into());
    }
    let mut result = GpuFrame {
        rgba,
        mask: Vec::new(),
        height: Vec::new(),
        heat: Vec::new(),
        normal_z: Vec::new(),
        adapter: hardware,
    };
    for chunk in fields.chunks_exact(16) {
        let read = |offset: usize| {
            f32::from_le_bytes(
                chunk[offset..offset + 4]
                    .try_into()
                    .expect("valid four bytes"),
            )
        };
        let h = read(0);
        let heat = read(4);
        let mask = read(8);
        let normal_z = read(12);
        if !h.is_finite() || !heat.is_finite() || !mask.is_finite() || !normal_z.is_finite() {
            return Err("GPU returned a non-finite material field".into());
        }
        result.height.push(h);
        result.heat.push(heat);
        result.mask.push(u32::from(mask > 0.5));
        result.normal_z.push(normal_z);
    }
    Ok(result)
}
