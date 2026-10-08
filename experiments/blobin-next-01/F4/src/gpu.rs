//! GPU F4 offscreen effects: one bounded Vulkan compute dispatch per declared pass.
//! F2 owns topology; no GPU output may mint or mutate scene identity.
use crate::{Pass, Plan};
use blobin_f2_topology::{cpu_mask, measure_mask, Scene, Topology};
use blobin_f3_material_height::{gpu as f3gpu, Params};
use std::{sync::mpsc, time::Duration};
use wgpu::util::DeviceExt;

#[derive(Default)]
pub struct GpuTemporal {
    previous: Option<Vec<u8>>,
    digest: Option<u64>,
    pub resets: usize,
}
impl GpuTemporal {
    pub fn reset(&mut self) {
        self.previous = None;
        self.digest = None;
        self.resets += 1;
    }
    pub fn len(&self) -> usize {
        self.previous.as_ref().map_or(0, Vec::len)
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
pub struct GpuOutput {
    pub rgba: Vec<u8>,
    pub topology: Topology,
    pub digest: u64,
    pub adapter: String,
    pub dispatches: usize,
    pub temporal_bytes: usize,
}
fn render_buffer(device: &wgpu::Device, name: &'static str, size: u64) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(name),
        size,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    })
}
pub fn process(
    scene: Scene,
    params: Params,
    plan: &Plan,
    temporal: &mut GpuTemporal,
) -> Result<GpuOutput, String> {
    plan.validate(scene)?;
    params.validate()?;
    // F3 actual GPU computes the base material source and numeric analytic masks.
    let source = f3gpu::render(scene, params)?;
    let cpu_mask = cpu_mask(scene)?;
    if source.mask.len() != cpu_mask.len()
        || !source
            .mask
            .iter()
            .zip(&cpu_mask)
            .all(|(a, b)| *a == u32::from(*b))
    {
        return Err("F4 F3 MATERIAL MASK MISMATCH".into());
    }
    let topology = measure_mask(&cpu_mask, scene.width(), scene.height())?;
    if topology != scene.layout.expected_topology() {
        return Err("F4 GPU INPUT TOPOLOGY RED".into());
    }
    let digest = scene.digest();
    if plan.reduced_motion || temporal.digest != Some(digest) {
        temporal.reset();
    }
    if plan.passes.is_empty() {
        return Ok(GpuOutput {
            rgba: source.rgba,
            topology,
            digest,
            adapter: source.adapter,
            dispatches: 0,
            temporal_bytes: 0,
        });
    }
    let desc = wgpu::InstanceDescriptor {
        backends: wgpu::Backends::VULKAN,
        ..wgpu::InstanceDescriptor::new_without_display_handle()
    };
    let instance = wgpu::Instance::new(desc);
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: None,
        ..Default::default()
    }))
    .map_err(|e| format!("F4 GPU ADAPTER UNAVAILABLE: {e}"))?;
    let info = adapter.get_info();
    let label = format!("{} / {} / {:?}", info.name, info.driver, info.backend);
    let (device, queue) =
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
            .map_err(|e| format!("F4 GPU DEVICE UNAVAILABLE: {e}"))?;
    let len = scene.width() as u64 * scene.height() as u64 * 4;
    let original = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("f4-original-f3"),
        contents: &source.rgba,
        usage: wgpu::BufferUsages::STORAGE,
    });
    let mut mask_bytes = Vec::with_capacity(len as usize);
    for m in cpu_mask {
        mask_bytes.extend_from_slice(&(m as u32).to_le_bytes());
    }
    let mask = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("f4-semantic-mask"),
        contents: &mask_bytes,
        usage: wgpu::BufferUsages::STORAGE,
    });
    let has_history = temporal
        .previous
        .as_ref()
        .is_some_and(|v| v.len() == len as usize);
    let history = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("f4-history"),
        contents: temporal
            .previous
            .as_deref()
            .filter(|x| x.len() == len as usize)
            .unwrap_or(&source.rgba),
        usage: wgpu::BufferUsages::STORAGE,
    });
    let a = render_buffer(&device, "f4-ping", len);
    let b = render_buffer(&device, "f4-pong", len);
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("f4-post-graph"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/f4.wgsl").into()),
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("f4-explicit-compute-pass"),
        layout: None,
        module: &shader,
        entry_point: Some("main"),
        compilation_options: Default::default(),
        cache: None,
    });
    let layout = pipeline.get_bind_group_layout(0);
    let mut controls = Vec::new();
    let mut bindings = Vec::new();
    // Uniform and bind-group ownership must survive until encoder submission.
    for (index, pass) in plan.passes.iter().enumerate() {
        let code = match pass {
            Pass::Shadow => 0u32,
            Pass::Blur => 1,
            Pass::Bloom => 2,
            Pass::Echo => 3,
        };
        let mut raw = Vec::with_capacity(32);
        for value in [
            scene.width(),
            scene.height(),
            0,
            0,
            code,
            plan.blur_radius as u32,
            u32::from(has_history),
            u32::from(plan.reduced_motion),
        ] {
            raw.extend_from_slice(&value.to_le_bytes());
        }
        let uniform = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("f4-pass-contract"),
            contents: &raw,
            usage: wgpu::BufferUsages::UNIFORM,
        });
        controls.push(uniform);
        let input = if index == 0 {
            &original
        } else if index % 2 == 1 {
            &a
        } else {
            &b
        };
        let output = if index % 2 == 0 { &a } else { &b };
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("f4-pass-bindings"),
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: controls[index].as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: original.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: input.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: output.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: mask.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: history.as_entire_binding(),
                },
            ],
        });
        bindings.push(group);
    }
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("f4-effect-graph"),
    });
    for (index, group) in bindings.iter().enumerate() {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("f4-typed-node"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, group, &[]);
        pass.dispatch_workgroups(scene.width().div_ceil(8), scene.height().div_ceil(8), 1);
        drop(pass);
        debug_assert!(index < 4);
    }
    let last = if plan.passes.len() % 2 == 1 { &a } else { &b };
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("f4-effect-readback"),
        size: len,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    encoder.copy_buffer_to_buffer(last, 0, &readback, 0, len);
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
        .map_err(|e| format!("F4 GPU POLL FAILED: {e}"))?;
    rx.recv_timeout(Duration::from_secs(15))
        .map_err(|e| format!("F4 GPU MAP TIMED OUT: {e}"))?
        .map_err(|e| format!("F4 GPU MAP FAILED: {e}"))?;
    let rgba = slice
        .get_mapped_range()
        .map_err(|e| e.to_string())?
        .to_vec();
    readback.unmap();
    if rgba.len() != len as usize {
        return Err("F4 GPU INVALID READBACK LENGTH".into());
    }
    if plan.passes.contains(&Pass::Echo) && !plan.reduced_motion {
        temporal.previous = Some(rgba.clone());
        temporal.digest = Some(digest);
    } else {
        temporal.previous = None;
        temporal.digest = None;
    }
    if temporal.len() > len as usize {
        return Err("F4 GPU TEMPORAL BUDGET EXCEEDED".into());
    }
    Ok(GpuOutput {
        rgba,
        topology,
        digest,
        adapter: label,
        dispatches: bindings.len(),
        temporal_bytes: temporal.len(),
    })
}
