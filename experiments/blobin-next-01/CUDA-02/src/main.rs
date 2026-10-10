use blobin_cuda02_provider::{
    compare, cpu, evaluate, generate_scene, Backend, Request, SceneAuthority,
};
use blobin_f2_topology::Scene;
use std::{env, path::PathBuf, time::Instant};

fn parse_backend(s: &str) -> Result<Backend, String> {
    match s {
        "cpu" => Ok(Backend::Cpu),
        "vulkan" => Ok(Backend::Vulkan),
        "cuda" => Ok(Backend::Cuda),
        "auto" => Ok(Backend::Auto),
        _ => Err("CUDA02 UNKNOWN BACKEND".into()),
    }
}
fn run() -> Result<(), String> {
    let mut backend = "all".to_string();
    let mut count = 64usize;
    let mut dpr = 1u32;
    let mut library: Option<PathBuf> = None;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--backend" => backend = args.next().ok_or("CUDA02 MISSING BACKEND")?,
            "--count" => {
                count = args
                    .next()
                    .ok_or("CUDA02 MISSING COUNT")?
                    .parse()
                    .map_err(|_| "CUDA02 COUNT PARSE")?
            }
            "--dpr" => {
                dpr = args
                    .next()
                    .ok_or("CUDA02 MISSING DPR")?
                    .parse()
                    .map_err(|_| "CUDA02 DPR PARSE")?
            }
            "--cuda-lib" => {
                library = Some(PathBuf::from(args.next().ok_or("CUDA02 MISSING LIBRARY")?))
            }
            "--help" => {
                println!("CUDA02 --backend all|cpu|vulkan|cuda|auto --count 16|64|256 --dpr 1|2 --cuda-lib /absolute/libblobin_cuda02.so");
                return Ok(());
            }
            _ => return Err(format!("CUDA02 UNKNOWN ARGUMENT {arg}")),
        }
    }
    if !matches!(backend.as_str(), "all" | "cpu" | "vulkan" | "cuda" | "auto") {
        return Err("CUDA02 BAD MODE".into());
    }
    if !(1..=256).contains(&count) || !matches!(dpr, 1 | 2) {
        return Err("CUDA02 OUTSIDE LAB SCENE BUDGET".into());
    }
    let scene = Scene {
        dpr,
        ..Scene::default()
    };
    let req = Request::from_f2(scene, generate_scene(count))?;
    let authority = SceneAuthority::from(scene);
    let begin = Instant::now();
    let oracle = cpu(&req)?;
    let cpu_ms = begin.elapsed().as_secs_f64() * 1000.0;
    let mut results = 0usize;
    let choices = if backend == "all" {
        vec![Backend::Cpu, Backend::Vulkan, Backend::Cuda]
    } else {
        vec![parse_backend(&backend)?]
    };
    for choice in choices {
        let now = Instant::now();
        let frame = evaluate(&req, choice, library.as_deref())?;
        let elapsed_ms = now.elapsed().as_secs_f64() * 1000.0;
        if frame.authority != authority {
            return Err("CUDA02 F2 AUTHORITY MUTATED".into());
        }
        let metrics = compare(&oracle, &frame.samples)?;
        let selected = match frame.backend {
            Backend::Cpu => "cpu",
            Backend::Vulkan => "vulkan",
            Backend::Cuda => "cuda",
            Backend::Auto => return Err("CUDA02 AUTO CANNOT BE COMPUTE BACKEND".into()),
        };
        println!("CUDA02_CASE_PASS requested={backend} backend={selected} primitives={count} dpr={dpr} pixels={} authority_epoch={} authority_digest={:016x} left_id={} right_id={} max_field={:.9} max_normal={:.9} max_coverage={:.9} mismatches={} boundary_mismatches={} cpu_oracle_ms={:.6} backend_end_to_end_ms={:.6}",
            frame.samples.len(),authority.epoch,authority.semantic_digest,authority.left_id,
            authority.right_id,metrics.max_distance,metrics.max_normal,metrics.max_coverage,
            metrics.occupancy_mismatch,metrics.near_boundary_mismatch,cpu_ms,elapsed_ms);
        results += 1;
    }
    // Causal negative: oracle must reject an altered result.
    let mut corrupt = oracle.clone();
    corrupt[0].distance += 0.25;
    if compare(&oracle, &corrupt).is_ok() {
        return Err("CUDA02 PARITY FALSIFIER FAILED".into());
    }
    println!("CUDA02_PASS requested={backend} primitives={count} dpr={dpr} verified_providers={results} negative_numeric=PASS f2_authority=PASS");
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("CUDA02_ERROR {e}");
        std::process::exit(1);
    }
}
