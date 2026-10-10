use blobin_cp01_hybrid::{gltf::import_glb, Vec3};
use std::{error::Error, fs::File, io::BufWriter, path::Path, sync::mpsc, time::Duration};
use wgpu::util::DeviceExt;

const W: u32 = 640;
const H: u32 = 432;
const N: usize = 1024;

fn points() -> Vec<[f32; 4]> {
    let mut r = 0x928fd1a37bb046c5_u64;
    let mut out = Vec::with_capacity(N);
    for _ in 0..N {
        let mut v = [0.0f32; 4];
        for x in v.iter_mut().take(3) {
            r = r
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            *x = ((r >> 11) as f64 / ((1u64 << 53) as f64) * 2.2 - 0.6) as f32;
        }
        out.push(v);
    }
    out
}
fn map_read(
    device: &wgpu::Device,
    buffer: &wgpu::Buffer,
    size: u64,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let (tx, rx) = mpsc::channel();
    buffer
        .slice(0..size)
        .map_async(wgpu::MapMode::Read, move |v| {
            let _ = tx.send(v);
        });
    device.poll(wgpu::PollType::Wait {
        submission_index: None,
        timeout: Some(Duration::from_secs(30)),
    })?;
    rx.recv_timeout(Duration::from_secs(30))??;
    let view = buffer.slice(0..size).get_mapped_range()?;
    let out = view.to_vec();
    drop(view);
    buffer.unmap();
    Ok(out)
}
fn main() -> Result<(), Box<dyn Error>> {
    pollster::block_on(run())
}
async fn run() -> Result<(), Box<dyn Error>> {
    let path = std::env::args().nth(1).ok_or("expected output PNG path")?;
    let surface = import_glb(include_bytes!(
        "../../../CP01-HYBRID/fixtures/tetra-indexed.glb"
    ))
    .map_err(|e| format!("untrusted glb: {e:?}"))?;
    let solid = surface
        .try_certify_convex()
        .map_err(|e| format!("unqualified convex geometry: {e:?}"))?;
    let mut geo = Vec::<[f32; 4]>::new();
    for t in &solid.mesh().triangles {
        let points = t.map(|index| solid.mesh().vertices[index as usize]);
        for p in points {
            geo.push([p.x as f32, p.y as f32, p.z as f32, 0.0]);
        }
    }
    if geo.len() != 12 {
        return Err("expected 4 triangle faces".into());
    }
    for t in &solid.mesh().triangles {
        let points = t.map(|index| solid.mesh().vertices[index as usize]);
        let n = points[1].minus(points[0]).cross(points[2].minus(points[0]));
        let n = n.scale(1.0 / n.norm());
        geo.push([
            n.x as f32,
            n.y as f32,
            n.z as f32,
            (-n.dot(points[0])) as f32,
        ]);
    }
    let samples = points();
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
    if info.backend != wgpu::Backend::Vulkan || info.vendor != 0x10de {
        return Err("physical reference Vulkan NVIDIA required".into());
    }
    println!(
        "GPU01_ACTUAL_ADAPTER name={} driver={} backend={:?} vendor=0x{:x}",
        info.name, info.driver, info.backend, info.vendor
    );
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor::default())
        .await?;
    let meshbuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("CERTIFIED_imported_tetra_triangles_and_planes"),
        contents: bytemuck::cast_slice(&geo),
        usage: wgpu::BufferUsages::STORAGE,
    });
    let samplesbuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("3D_independent_samples"),
        contents: bytemuck::cast_slice(&samples),
        usage: wgpu::BufferUsages::STORAGE,
    });
    let samples_len = (N * 16) as u64;
    let pixels_len = (W * H * 4) as u64;
    let samples_output = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("gpu01-computed-sdf-samples"),
        size: samples_len,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let frame = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("gpu01-physical-vulkan-rendered-surface"),
        size: pixels_len,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let staging_s = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("gpu01-readback-sample"),
        size: samples_len,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let staging_p = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("gpu01-readback-pixels"),
        size: pixels_len,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let entries = [
        (0u32, wgpu::ShaderStages::COMPUTE, true),
        (1u32, wgpu::ShaderStages::COMPUTE, true),
        (2u32, wgpu::ShaderStages::COMPUTE, false),
        (3u32, wgpu::ShaderStages::COMPUTE, false),
    ];
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("mixed-triangle-sdf-and-pixels"),
        entries: &entries.map(
            |(binding, visibility, read_only)| wgpu::BindGroupLayoutEntry {
                binding,
                visibility,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ),
    });
    let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("signed-triangle-mesh-live-GPU"),
        layout: &layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: meshbuf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: samplesbuf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: samples_output.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: frame.as_entire_binding(),
            },
        ],
    });
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("actual_gpu_signed_triangle_surface_render"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../surface.wgsl").into()),
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("signed-mesh-render-pipeline-layout"),
        bind_group_layouts: &[Some(&layout)],
        immediate_size: 0,
    });
    let field_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("exact-triangle-SDF-GPU-parity"),
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: Some("field_samples"),
        compilation_options: Default::default(),
        cache: None,
    });
    let frame_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("GPU01-raymarched-render-output"),
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: Some("render"),
        compilation_options: Default::default(),
        cache: None,
    });
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("GPU01-compute-real-3D"),
    });
    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("GPU01 compute signed triangle distances"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&field_pipeline);
        pass.set_bind_group(0, &bind, &[]);
        pass.dispatch_workgroups(N.div_ceil(64) as u32, 1, 1);
    }
    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("GPU01 render actual hybrid 3D surface"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&frame_pipeline);
        pass.set_bind_group(0, &bind, &[]);
        pass.dispatch_workgroups(W.div_ceil(8), H.div_ceil(8), 1);
    }
    encoder.copy_buffer_to_buffer(&samples_output, 0, &staging_s, 0, samples_len);
    encoder.copy_buffer_to_buffer(&frame, 0, &staging_p, 0, pixels_len);
    queue.submit([encoder.finish()]);
    let sample_bytes = map_read(&device, &staging_s, samples_len)?;
    let gpu_samples = bytemuck::cast_slice::<u8, [f32; 4]>(&sample_bytes);
    let mut max_err = 0f64;
    for (pt, computed) in samples.iter().zip(gpu_samples.iter()) {
        let p = Vec3::new(pt[0] as f64, pt[1] as f64, pt[2] as f64);
        let actual_mesh = solid
            .signed_distance(p)
            .map_err(|e| format!("CPU signed mesh: {e:?}"))?;
        let actual_sphere = p.minus(Vec3::new(0.42, 0.34, 0.47)).norm() - 0.38;
        let expected = [actual_mesh, actual_sphere, actual_mesh.max(-actual_sphere)];
        for (j, &v) in expected.iter().enumerate() {
            let delta = (computed[j] as f64 - v).abs();
            max_err = max_err.max(delta);
            if delta > 0.0002 {
                return Err(format!(
                    "unexpected GPU/CPU signed mesh mismatch index={j} err={delta} point={p:?}"
                )
                .into());
            }
        }
    }
    println!("GPU01_EUCLIDEAN_TRIANGLE_CPU_PARITY_PASS points={N} channels=3 comparisons={} max_abs_err={max_err:.12e}",3*N);
    let pixel_bytes = map_read(&device, &staging_p, pixels_len)?;
    let mut encoder_png = png::Encoder::new(BufWriter::new(File::create(Path::new(&path))?), W, H);
    encoder_png.set_color(png::ColorType::Rgba);
    encoder_png.set_depth(png::BitDepth::Eight);
    let mut writer = encoder_png.write_header()?;
    writer.write_image_data(&pixel_bytes)?;
    writer.finish()?;
    println!("GPU01_RENDERED_SURFACE_PNG width={W} height={H} path={path}");
    println!("GPU01_NOT_CP02_QUALIFIED: offscreen Vulkan render, NO compositor presentation/60FPS");
    Ok(())
}
