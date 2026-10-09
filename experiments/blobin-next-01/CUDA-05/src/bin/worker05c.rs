//! Sacrificial GPU worker. Every invocation uses a fresh process and GPU context.
use blobin_cuda02_provider::{compare, cpu, generate_scene, Request};
use blobin_cuda04_bridge::{provider, resident::ResidentBridge, Device};
use blobin_f2_topology::Scene;
use std::{env, path::PathBuf, time::Duration};

fn hex(s: &str) -> Result<[u8; 32], String> {
    if s.len() != 64 {
        return Err("CUDA05C HASH REQUIRED".into());
    }
    let mut result = [0u8; 32];
    for (i, pair) in s.as_bytes().chunks_exact(2).enumerate() {
        result[i] = u8::from_str_radix(std::str::from_utf8(pair).map_err(|e| e.to_string())?, 16)
            .map_err(|_| "CUDA05C INVALID HEX")?;
    }
    Ok(result)
}
fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let mut backend = None;
    let mut path = None;
    let mut approved = None;
    let mut fault = None;
    while let Some(flag) = args.next() {
        let v = args.next().ok_or("CUDA05C FLAG VALUE REQUIRED")?;
        match flag.as_str() {
            "--backend" => {
                backend = Some(match v.as_str() {
                    "cuda" => Device::Cuda,
                    "vulkan" => Device::Vulkan,
                    _ => return Err("CUDA05C INVALID BACKEND".into()),
                })
            }
            "--cuda-lib" => path = Some(PathBuf::from(v)),
            "--sha256" => approved = Some(hex(&v)?),
            "--inject-fault" => fault = Some(v),
            _ => return Err("CUDA05C UNKNOWN FLAG".into()),
        }
    }
    let backend = backend.ok_or("CUDA05C BACKEND REQUIRED")?;
    if fault.as_deref() == Some("hang") {
        std::thread::sleep(Duration::from_secs(30));
        return Err("CUDA05C INJECTED HANG ENDED".into());
    }
    if fault.as_deref() == Some("crash") {
        // Kill only this sacrificial process; never the graphical session.
        unsafe { libc::raise(libc::SIGKILL) };
        return Err("CUDA05C INJECTED CRASH SURVIVED".into());
    }
    if fault.is_some() {
        return Err("CUDA05C UNKNOWN INJECTION".into());
    }
    let lib = if backend == Device::Cuda {
        let lib = path.as_deref().ok_or("CUDA05C CUDA LIB REQUIRED")?;
        let hash = approved.ok_or("CUDA05C APPROVAL REQUIRED")?;
        if provider::inspect(lib)?.fingerprint.sha256 != hash {
            return Err("CUDA05C UNAPPROVED CUDA PROVIDER".into());
        }
        Some(lib)
    } else {
        None
    };
    let req = Request::from_f2(Scene::default(), generate_scene(16))?;
    let expected = cpu(&req)?;
    let mut resident = ResidentBridge::new_managed(backend, approved)?;
    resident.report_device_recovered()?;
    let rendered = resident.evaluate_observed(&req, lib)?;
    if rendered.output.cache_hit || rendered.output.selected != backend {
        return Err("CUDA05C INVALID EXECUTION PROVENANCE".into());
    }
    compare(&expected, &rendered.output.samples)?;
    println!(
        "CUDA05C_WORKER_PASS backend={backend:?} pixels={} parity=PASS cache_miss=PASS",
        rendered.output.samples.len()
    );
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("CUDA05C_WORKER_ERROR {e}");
        std::process::exit(1);
    }
}
