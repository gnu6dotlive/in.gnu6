use crate::FrameIr;
use std::sync::mpsc;
use std::time::Duration;
use wgpu::util::DeviceExt;

/// Offscreen Vulkan compute renderer. Does not own or mutate scene identity.
pub fn render(frame: &FrameIr) -> Result<(Vec<u8>, String), String> {
    frame.validate()?;
    let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
    descriptor.backends = wgpu::Backends::VULKAN;
    let instance = wgpu::Instance::new(descriptor);
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: None,
        ..Default::default()
    }))
    .map_err(|e| format!("GPU adapter unavailable: {e}"))?;
    let info = adapter.get_info();
    let (device, queue) =
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
            .map_err(|e| format!("GPU device unavailable: {e}"))?;
    let label = format!(
        "{} / {} / {:?} / {:?}",
        info.name, info.driver, info.backend, info.device_type
    );
    let mut ir_bytes = Vec::with_capacity(32);
    for x in [
        frame.pixel_width(),
        frame.pixel_height(),
        frame.logical_width,
        frame.logical_height,
    ] {
        ir_bytes.extend_from_slice(&x.to_le_bytes());
    }
    for x in [
        frame.dpr as f32,
        frame.phase,
        if frame.focused { 1.0 } else { 0.0 },
        0.0f32,
    ] {
        ir_bytes.extend_from_slice(&x.to_le_bytes());
    }
    let frame_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("f1-frame-ir-read-only"),
        contents: &ir_bytes,
        usage: wgpu::BufferUsages::UNIFORM,
    });
    let len = (frame.pixel_width() as u64) * (frame.pixel_height() as u64) * 4;
    let output = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("f1-gpu-rgba"),
        size: len,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("f1-readback"),
        size: len,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("f1-radial-contour"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/f1.wgsl").into()),
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("f1-offscreen-compute"),
        layout: None,
        module: &shader,
        entry_point: Some("main"),
        compilation_options: Default::default(),
        cache: None,
    });
    let layout = pipeline.get_bind_group_layout(0);
    let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("f1-snapshot-bindings"),
        layout: &layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: frame_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: output.as_entire_binding(),
            },
        ],
    });
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("f1-single-frame"),
    });
    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("f1-field-evaluator"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bind, &[]);
        pass.dispatch_workgroups(
            frame.pixel_width().div_ceil(8),
            frame.pixel_height().div_ceil(8),
            1,
        );
    }
    encoder.copy_buffer_to_buffer(&output, 0, &readback, 0, len);
    queue.submit(Some(encoder.finish()));
    let slice = readback.slice(..);
    let (tx, rx) = mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = tx.send(result);
    });
    device
        .poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: Some(Duration::from_secs(15)),
        })
        .map_err(|e| format!("GPU poll failure: {e}"))?;
    rx.recv_timeout(Duration::from_secs(15))
        .map_err(|e| format!("GPU map callback timeout: {e}"))?
        .map_err(|e| format!("GPU readback map failure: {e}"))?;
    let bytes = slice
        .get_mapped_range()
        .map_err(|e| format!("GPU readback unavailable: {e}"))?
        .to_vec();
    readback.unmap();
    if bytes.len() != len as usize {
        return Err("incorrect GPU readback length".into());
    }
    Ok((bytes, label))
}
