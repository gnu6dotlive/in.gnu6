use blobin_cuda02_provider::{compare, cpu, generate_scene, Request};
use blobin_cuda04_bridge::{geometry_id, resident::ResidentBridge, Device, Output};
use blobin_f2_topology::Scene;
use std::{
    env,
    path::{Path, PathBuf},
    time::Instant,
};

fn verify(req: &Request, out: &Output, backend: Device, hit: bool) -> Result<(), String> {
    if out.selected != backend
        || out.cache_hit != hit
        || out.scene != req.authority()
        || out.geometry != geometry_id(req)?
    {
        return Err("CUDA04C PROVENANCE OR CACHE CONTRACT".into());
    }
    compare(&cpu(req)?, &out.samples)?;
    Ok(())
}
fn quantiles(values: &[f64]) -> (f64, f64) {
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let rank = |p: f64| sorted[((sorted.len() as f64 * p).ceil() as usize).saturating_sub(1)];
    (rank(0.5), rank(0.95))
}
fn eval(
    bridge: &mut ResidentBridge,
    req: &Request,
    backend: Device,
    lib: &Path,
    hit: bool,
) -> Result<f64, String> {
    let start = Instant::now();
    let output = bridge.evaluate(req, Some(lib))?;
    let elapsed = start.elapsed().as_secs_f64() * 1000.0;
    verify(req, &output, backend, hit)?;
    Ok(elapsed)
}
fn label(device: Device) -> &'static str {
    match device {
        Device::Cuda => "cuda",
        Device::Vulkan => "vulkan",
        Device::Cpu => "cpu",
    }
}
fn case(count: usize, dpr: u32, backend: Device, n: usize, lib: &Path) -> Result<(), String> {
    let scene = Scene {
        dpr,
        ..Scene::default()
    };
    let original = Request::from_f2(scene, generate_scene(count))?;
    let mut b = ResidentBridge::new(backend)?;
    let cold = eval(&mut b, &original, backend, lib, false)?;
    let bytes = b.resident_bytes();
    if bytes == 0 {
        return Err("CUDA04C ZERO RESIDENT BYTES".into());
    }
    let cached = eval(&mut b, &original, backend, lib, true)?;
    let mut prims = generate_scene(count);
    prims[0].radius += 0.08;
    let edit = Request::from_f2(scene, prims.clone())?;
    if original.authority() != edit.authority()
        || geometry_id(&original)? == geometry_id(&edit)?
        || compare(&cpu(&original)?, &cpu(&edit)?).is_ok()
    {
        return Err("CUDA04C STALE EDIT NOT OBSERVABLE".into());
    }
    let edited = eval(&mut b, &edit, backend, lib, false)?;
    if b.resident_bytes() != bytes {
        return Err("CUDA04C RESIDENT CAPACITY DRIFT".into());
    }
    eval(&mut b, &edit, backend, lib, true)?;
    let next_epoch = Request::from_f2(
        Scene {
            epoch: scene.epoch + 1,
            ..scene
        },
        prims,
    )?;
    if next_epoch.authority() == edit.authority()
        || geometry_id(&next_epoch)? != geometry_id(&edit)?
    {
        return Err("CUDA04C EPOCH SEPARATION FAILED".into());
    }
    eval(&mut b, &next_epoch, backend, lib, true)?;
    let mut timings = Vec::new();
    for i in 0..n {
        let mut p = generate_scene(count);
        p[0].radius += 0.082 + i as f32 * 0.0003;
        let req = Request::from_f2(scene, p)?;
        let ms = eval(&mut b, &req, backend, lib, false)?;
        if b.resident_bytes() != bytes {
            return Err("CUDA04C RESIDENT CAPACITY DRIFT".into());
        }
        println!(
            "CUDA04C_ITER_PASS backend={} count={count} dpr={dpr} index={i} host_ms={ms:.6}",
            label(backend)
        );
        timings.push(ms);
    }
    // Same pixel count, reversed dimensions: CUDA-03 alone cannot detect this shape change.
    let swapped = Scene {
        logical_width: scene.logical_height,
        logical_height: scene.logical_width,
        ..scene
    };
    let swapped_req = Request::from_f2(swapped, generate_scene(count))?;
    eval(&mut b, &swapped_req, backend, lib, false)?;
    if b.resident_bytes() != bytes {
        return Err("CUDA04C SAME-AREA TRANSPOSE CAPACITY DRIFT".into());
    }
    let resized = Scene {
        dpr: if dpr == 1 { 2 } else { 1 },
        ..scene
    };
    let req = Request::from_f2(resized, generate_scene(count))?;
    let rebuild = eval(&mut b, &req, backend, lib, false)?;
    if b.resident_bytes() == bytes {
        return Err("CUDA04C RESHAPE DID NOT REALLOCATE".into());
    }
    let (p50, p95) = quantiles(&timings);
    println!("CUDA04C_CASE_PASS backend={} count={count} dpr={dpr} iterations={n} cold_ms={cold:.6} cached_ms={cached:.6} edited_ms={edited:.6} update_p50_ms={p50:.6} update_p95_ms={p95:.6} rebuild_ms={rebuild:.6} resident_bytes={bytes} numeric=PASS stale=PASS epoch=PASS transpose=PASS rebuild=PASS",label(backend));
    b.invalidate();
    if b.resident_bytes() != 0 {
        return Err("CUDA04C INVALIDATE FAILED".into());
    }
    Ok(())
}
fn negatives(lib: &Path) -> Result<(), String> {
    let req = Request::from_f2(Scene::default(), generate_scene(16))?;
    let mut b = ResidentBridge::new(Device::Cuda)?;
    if b.evaluate(&req, None).is_ok() || b.resident_bytes() != 0 {
        return Err("CUDA04C ABSENT PROVIDER ACCEPTED".into());
    }
    verify(&req, &b.evaluate(&req, Some(lib))?, Device::Cuda, false)?;
    if b.evaluate(&req, Some(Path::new("/nonexistent/cuda04c/provider.so")))
        .is_ok()
        || b.resident_bytes() != 0
    {
        return Err("CUDA04C PROVIDER SWITCH FAILED".into());
    }
    verify(&req, &b.evaluate(&req, Some(lib))?, Device::Cuda, false)?;
    if b.evaluate(&req, None).is_ok() || b.resident_bytes() != 0 {
        return Err("CUDA04C PROVIDER REMOVAL FAILED".into());
    }
    println!("CUDA04C_NEGATIVE_PASS absent=PASS switched=PASS recovered=PASS revoked=PASS");
    Ok(())
}
fn run() -> Result<(), String> {
    let mut lib: Option<PathBuf> = None;
    let mut n = 8usize;
    let mut args = env::args().skip(1);
    while let Some(s) = args.next() {
        match s.as_str() {
            "--cuda-lib" => lib = Some(PathBuf::from(args.next().ok_or("CUDA04C LIB VALUE")?)),
            "--iterations" => {
                n = args
                    .next()
                    .ok_or("CUDA04C ITER VALUE")?
                    .parse()
                    .map_err(|_| "CUDA04C BAD ITER")?
            }
            other => return Err(format!("CUDA04C UNKNOWN FLAG {other}")),
        }
    }
    if !(8..=30).contains(&n) {
        return Err("CUDA04C ITERATION BOUNDS".into());
    }
    let lib = lib.ok_or("CUDA04C LIB REQUIRED")?;
    if !lib.is_absolute() || !lib.is_file() {
        return Err("CUDA04C LIB INVALID".into());
    }
    negatives(&lib)?;
    for count in [16, 64, 256] {
        for dpr in [1, 2] {
            for backend in [Device::Cuda, Device::Vulkan] {
                case(count, dpr, backend, n, &lib)?;
            }
        }
    }
    println!(
        "CUDA04C_ALL_PASS cases=12 iterations={} negatives=4",
        n * 12
    );
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("CUDA04C_ERROR {e}");
        std::process::exit(1);
    }
}
