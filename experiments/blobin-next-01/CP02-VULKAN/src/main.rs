use blobin_cp01_hybrid::gltf::import_glb;
use std::{error::Error, sync::mpsc, time::Duration};
use wgpu::util::DeviceExt;

const NUM: usize = 1024;

fn generate_points() -> Vec<[f32; 4]> {
    let mut rng = 0xbe00_9145_d3ce_38f1_u64;
    let mut points = Vec::with_capacity(NUM);
    for _ in 0..NUM {
        let mut p = [0.0; 4];
        for item in &mut p[0..3] {
            rng = rng
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let v = ((rng >> 11) as f64) / ((1u64 << 53) as f64);
            *item = (v * 2.0 - 0.5) as f32;
        }
        points.push(p);
    }
    points
}
fn main() -> Result<(), Box<dyn Error>> {
    pollster::block_on(run())
}
async fn run() -> Result<(), Box<dyn Error>> {
    let tetra = import_glb(include_bytes!(
        "../../CP01-HYBRID/fixtures/tetra-indexed.glb"
    ))
    .map_err(|e| format!("invalid GLB input: {e:?}"))?;
    // Explicit real imported convex topology certification, not wire-level trust.
    let tetra = tetra
        .try_certify_convex()
        .map_err(|e| format!("uncertified MeshSolid: {e:?}"))?;
    let m = tetra.mesh();
    if m.triangles.len() != 4 {
        return Err("fixture no longer tetra".into());
    }
    let mut planes = Vec::<[f32; 4]>::new();
    for [ia, ib, ic] in &m.triangles {
        let a = m.vertices[*ia as usize];
        let b = m.vertices[*ib as usize];
        let c = m.vertices[*ic as usize];
        let ab = b.minus(a);
        let ac = c.minus(a);
        let n = ab.cross(ac);
        let normalized = n.scale(1.0 / n.norm());
        planes.push([
            normalized.x as f32,
            normalized.y as f32,
            normalized.z as f32,
            (-normalized.dot(a)) as f32,
        ]);
    }
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::VULKAN,
        flags: Default::default(),
        memory_budget_thresholds: Default::default(),
        backend_options: Default::default(),
        display: None,
    });
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            ..Default::default()
        })
        .await?;
    let info = adapter.get_info();
    if info.backend != wgpu::Backend::Vulkan {
        return Err("actual Vulkan backend absent".into());
    }
    println!(
        "GPU_ACTUAL_ADAPTER name={} vendor=0x{:04x} device=0x{:04x} backend={:?} driver={}",
        info.name, info.vendor, info.device, info.backend, info.driver
    );
    if info.vendor != 0x10de {
        return Err("this prereg GPU fixture requires physical NVIDIA reference".into());
    }
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor::default())
        .await?;
    let points = generate_points();
    let points_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("real-3d-input-points"),
        contents: bytemuck::cast_slice(&points),
        usage: wgpu::BufferUsages::STORAGE,
    });
    let planes_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("cpu-certified-mesh-planes"),
        contents: bytemuck::cast_slice(&planes),
        usage: wgpu::BufferUsages::STORAGE,
    });
    let bytes = (NUM * std::mem::size_of::<[f32; 4]>()) as u64;
    let output = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("mixed-csg-output-device"),
        size: bytes,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("mixed-csg-readback"),
        size: bytes,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("mixed-csg-wgsl-compute"),
        source: wgpu::ShaderSource::Wgsl(include_str!("mixed_csg.wgsl").into()),
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("mixed-csg-compute"),
        layout: None,
        module: &shader,
        entry_point: Some("main"),
        compilation_options: Default::default(),
        cache: None,
    });
    let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("mixed-csg-bind"),
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: points_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: planes_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: output.as_entire_binding(),
            },
        ],
    });
    let mut commands = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("gpu00-encoder"),
    });
    {
        let mut pass = commands.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("actual-wgpu-mixed-csg"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bind, &[]);
        pass.dispatch_workgroups(NUM.div_ceil(64) as u32, 1, 1);
    }
    commands.copy_buffer_to_buffer(&output, 0, &readback, 0, bytes);
    let submission = queue.submit([commands.finish()]);
    let (tx, rx) = mpsc::channel();
    readback.map_async(wgpu::MapMode::Read, .., move |result| {
        let _ = tx.send(result);
    });
    device.poll(wgpu::PollType::Wait {
        submission_index: Some(submission),
        timeout: Some(Duration::from_secs(20)),
    })?;
    rx.recv_timeout(Duration::from_secs(20))??;
    let view = readback.get_mapped_range(..)?;
    let actual = bytemuck::cast_slice::<u8, [f32; 4]>(&view);
    let mut worst = 0.0f32;
    let mut mismatch = 0;
    for (i, point) in points.iter().enumerate() {
        let p = [point[0], point[1], point[2]];
        let dx = p[0] - 0.15;
        let dy = p[1] - 0.15;
        let dz = p[2] - 0.15;
        let sphere = (dx * dx + dy * dy + dz * dz).sqrt() - 0.30;
        let mut convex = -1e30f32;
        for plane in &planes {
            convex = convex.max(plane[0] * p[0] + plane[1] * p[1] + plane[2] * p[2] + plane[3]);
        }
        let expected = [
            sphere.min(convex),
            sphere.max(convex),
            sphere.max(-convex),
            convex.max(-sphere),
        ];
        for (k, &reference) in expected.iter().enumerate() {
            let diff = (actual[i][k] - reference).abs();
            worst = worst.max(diff);
            if diff > 1e-5 {
                mismatch += 1;
            }
        }
    }
    drop(view);
    readback.unmap();
    println!("GPU00_ACTUAL_VULKAN_MIXED_CSG_PASS points={NUM} channels=4 comparisons={} max_abs_err={worst:.9e} mismatches={mismatch}",NUM*4);
    if mismatch != 0 {
        return Err("GPU CPU field parity mismatch".into());
    }
    println!("GPU00_NOT_CP02_QUALIFIED: no renderer, CSG marching, Wayland presentation or 60FPS");
    Ok(())
}
