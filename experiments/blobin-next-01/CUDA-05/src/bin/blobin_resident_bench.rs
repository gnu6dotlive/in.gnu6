//! Research-only warm GPU session benchmark; not a supervised renderer.
use blobin_cuda02_provider::{Primitive, Request};
use blobin_cuda04_bridge::{provider, resident::ResidentBridge, Device};
use blobin_cuda05_ingress::frame_protocol::colorize;
use blobin_f2_topology::Scene;
use std::{env, path::Path, time::Instant};
fn run() -> Result<(), String> {
    let backend = env::args().nth(1).ok_or("backend cuda|vulkan missing")?;
    let gpu = match backend.as_str() {
        "cuda" => Device::Cuda,
        "vulkan" => Device::Vulkan,
        _ => return Err("GPU_ONLY".into()),
    };
    let lib = Path::new("/mnt/workbench/build/blobin-cuda04d/libblobin_cuda03.so");
    let approved = if gpu == Device::Cuda {
        Some(provider::inspect(lib)?.fingerprint.sha256)
    } else {
        None
    };
    let mut engine = ResidentBridge::new_managed(gpu, approved)?;
    engine.report_device_recovered()?;
    let mut times = Vec::new();
    let mut hashes = std::collections::HashSet::new();
    for frame in 0..48 {
        let f = frame as f32 / 48.0;
        let t = (f * std::f32::consts::TAU).sin();
        let scene = Scene {
            epoch: (frame + 1) as u64,
            logical_width: 320,
            logical_height: 224,
            dpr: 1,
            ..Scene::default()
        };
        let prim = vec![
            Primitive {
                x: -0.12 + t * 0.05,
                y: 0.02,
                radius: 0.22,
                op: 0,
            },
            Primitive {
                x: 0.14 - t * 0.04,
                y: 0.06,
                radius: 0.13,
                op: 3,
            },
            Primitive {
                x: 0.0,
                y: -0.07 + t * 0.04,
                radius: 0.04,
                op: 1,
            },
        ];
        let req = Request::from_f2(scene, prim)?;
        let started = Instant::now();
        let got =
            engine.evaluate_observed(&req, if gpu == Device::Cuda { Some(lib) } else { None })?;
        if got.output.cache_hit {
            return Err("INVALID_CACHED_ANIMATION_FRAME".into());
        }
        let rgba = colorize(&got.output.samples, "lit")?;
        if rgba.len() != 320 * 224 * 4 {
            return Err("INVALID_RGBA_LENGTH".into());
        }
        let millis = started.elapsed().as_secs_f64() * 1000.0;
        times.push(millis);
        let fingerprint = rgba.iter().fold(14695981039346656037u64, |h, b| {
            (h ^ (*b as u64)).wrapping_mul(1099511628211)
        });
        hashes.insert(fingerprint);
    }
    let first = times[0];
    let mut warm = times[1..].to_vec();
    warm.sort_by(f64::total_cmp);
    let median = warm[warm.len() / 2];
    if hashes.len() < 24 {
        return Err("FRAMES_NOT_VISUALLY_DISTINCT".into());
    }
    println!("BLOBIN_RESIDENT_BENCH_PASS backend={backend} frames=48 unique_rgba={} first_ms={first:.3} warm_median_ms={median:.3} warm_max_ms={:.3} hypothetical_kernel_plus_color_fps={:.1} scope=RESEARCH_NOT_SUPERVISED",hashes.len(),warm[warm.len()-1],1000.0/median);
    Ok(())
}
fn main() {
    if let Err(err) = run() {
        eprintln!("RESIDENT_BENCH_FAIL {err}");
        std::process::exit(2)
    }
}
