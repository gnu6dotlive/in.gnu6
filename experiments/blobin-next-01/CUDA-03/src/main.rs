use blobin_cuda02_provider::{compare, cpu, generate_scene, Request};
use blobin_cuda03_resident::{cuda, summary, vulkan};
use blobin_f2_topology::Scene;
use std::{env, path::PathBuf, time::Instant};

fn run() -> Result<(), String> {
    let mut count = 64usize;
    let mut dpr = 1u32;
    let mut iterations = 20usize;
    let mut lib: Option<PathBuf> = None;
    let mut args = env::args().skip(1);
    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--count" => {
                count = args
                    .next()
                    .ok_or("CUDA03 MISSING COUNT")?
                    .parse()
                    .map_err(|_| "CUDA03 BAD COUNT")?
            }
            "--dpr" => {
                dpr = args
                    .next()
                    .ok_or("CUDA03 MISSING DPR")?
                    .parse()
                    .map_err(|_| "CUDA03 BAD DPR")?
            }
            "--iterations" => {
                iterations = args
                    .next()
                    .ok_or("CUDA03 MISSING ITERATIONS")?
                    .parse()
                    .map_err(|_| "CUDA03 BAD ITERATIONS")?
            }
            "--cuda-lib" => lib = Some(PathBuf::from(args.next().ok_or("CUDA03 MISSING LIB")?)),
            "--help" => {
                println!(
                    "--count 16|64|256 --dpr 1|2 --iterations 8..40 --cuda-lib /absolute/path.so"
                );
                return Ok(());
            }
            other => return Err(format!("CUDA03 UNKNOWN ARGUMENT {other}")),
        }
    }
    if !(1..=256).contains(&count) || !matches!(dpr, 1 | 2) || !(8..=40).contains(&iterations) {
        return Err("CUDA03 LAB BUDGET".into());
    }
    let lib = lib.ok_or("CUDA03 EXPLICIT CUDA LIBRARY REQUIRED")?;
    let scene = Scene {
        dpr,
        ..Scene::default()
    };
    let request = Request::from_f2(scene, generate_scene(count))?;
    let started = Instant::now();
    let reference = cpu(&request)?;
    let cpu_ms = started.elapsed().as_secs_f64() * 1000.0;
    let init_cuda = Instant::now();
    let mut cuda = cuda::Resident::new(&request, &lib)?;
    let cuda_init_ms = init_cuda.elapsed().as_secs_f64() * 1000.0;
    let init_vulkan = Instant::now();
    let mut vulkan = vulkan::Resident::new(&request)?;
    let vk_init_ms = init_vulkan.elapsed().as_secs_f64() * 1000.0;
    for _ in 0..4 {
        cuda.launch()?;
        vulkan.launch()?;
    }
    let mut cuda_events = Vec::new();
    let mut cuda_host = Vec::new();
    for _ in 0..iterations {
        let start = Instant::now();
        let elapsed = cuda.launch()?;
        cuda_host.push(start.elapsed().as_secs_f64() * 1000.0);
        cuda_events.push(elapsed as f64);
    }
    let mut vk_host = Vec::new();
    for _ in 0..iterations {
        let start = Instant::now();
        vulkan.launch()?;
        vk_host.push(start.elapsed().as_secs_f64() * 1000.0);
    }
    let start = Instant::now();
    let cuda_frame = cuda.readback()?;
    let cuda_copy_ms = start.elapsed().as_secs_f64() * 1000.0;
    let start = Instant::now();
    let vk_frame = vulkan.readback()?;
    let vk_copy_ms = start.elapsed().as_secs_f64() * 1000.0;
    let c = compare(&reference, &cuda_frame)?;
    let v = compare(&reference, &vk_frame)?;
    let (event_avg, event_p50, event_p95) = summary(&cuda_events)?;
    let (_, cuda_host_p50, cuda_host_p95) = summary(&cuda_host)?;
    let (_, vk_host_p50, vk_host_p95) = summary(&vk_host)?;
    println!("CUDA03_CASE_PASS backend=cuda count={count} dpr={dpr} iterations={iterations} pixels={} digest={:016x} init_ms={cuda_init_ms:.5} cpu_once_ms={cpu_ms:.5} gpu_event_mean_ms={event_avg:.6} gpu_event_p50_ms={event_p50:.6} gpu_event_p95_ms={event_p95:.6} host_p50_ms={cuda_host_p50:.6} host_p95_ms={cuda_host_p95:.6} readback_ms={cuda_copy_ms:.6} resident_bytes={} max_field={:.9} max_normal={:.9} max_coverage={:.9} mismatches={}",
        reference.len(),request.authority().semantic_digest,cuda.resident_bytes(),
        c.max_distance,c.max_normal,c.max_coverage,c.occupancy_mismatch);
    println!("CUDA03_CASE_PASS backend=vulkan count={count} dpr={dpr} iterations={iterations} pixels={} digest={:016x} init_ms={vk_init_ms:.5} cpu_once_ms={cpu_ms:.5} gpu_event_mean_ms=NA gpu_event_p50_ms=NA gpu_event_p95_ms=NA host_p50_ms={vk_host_p50:.6} host_p95_ms={vk_host_p95:.6} readback_ms={vk_copy_ms:.6} resident_bytes={} max_field={:.9} max_normal={:.9} max_coverage={:.9} mismatches={}",
        reference.len(),request.authority().semantic_digest,vulkan.resident_bytes(),
        v.max_distance,v.max_normal,v.max_coverage,v.occupancy_mismatch);
    let mut modified = generate_scene(count);
    modified[0].radius += 0.080;
    let updated = Request::from_f2(scene, modified)?;
    if updated.authority() != request.authority() {
        return Err("CUDA03 AUTHORITY CHANGED ON GEOMETRY EDIT".into());
    }
    let updated_reference = cpu(&updated)?;
    if compare(&reference, &updated_reference).is_ok() {
        return Err("CUDA03 NEGATIVE STALE GEOMETRY NOT OBSERVABLE".into());
    }
    let cuda_resident_bytes = cuda.resident_bytes();
    let vulkan_resident_bytes = vulkan.resident_bytes();
    cuda.update(&updated)?;
    vulkan.update(&updated)?;
    cuda.launch()?;
    vulkan.launch()?;
    let edited_cuda = compare(&updated_reference, &cuda.readback()?)?;
    let edited_vk = compare(&updated_reference, &vulkan.readback()?)?;
    if cuda.resident_bytes() != cuda_resident_bytes
        || vulkan.resident_bytes() != vulkan_resident_bytes
    {
        return Err("CUDA03 UPDATE CHANGED RESIDENT BUFFER CAPACITY".into());
    }
    println!("CUDA03_UPDATE_PASS count={count} dpr={dpr} digest={:016x} cuda_field={:.9} vulkan_field={:.9} allocation_bytes_unchanged=PASS stale_falsifier=PASS",
        request.authority().semantic_digest,edited_cuda.max_distance,edited_vk.max_distance);
    println!("CUDA03_PASS count={count} dpr={dpr} iterations={iterations} providers=3 gpu_buffers_resident=PASS numeric_parity=PASS");
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("CUDA03_ERROR {error}");
        std::process::exit(1);
    }
}
