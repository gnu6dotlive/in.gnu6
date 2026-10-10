//! CUDA-04D bounded real GPU qualification + injected control-plane revocations.
use blobin_cuda02_provider::{compare, cpu, generate_scene, Request};
use blobin_cuda04_bridge::{
    health::HealthState, provider::inspect, resident::ResidentBridge, Device,
};
use blobin_f2_topology::Scene;
use std::{
    env, fs,
    path::{Path, PathBuf},
    time::Instant,
};

fn check(
    b: &mut ResidentBridge,
    req: &Request,
    provider: Option<&Path>,
    expected_hit: bool,
) -> Result<f64, String> {
    let start = Instant::now();
    let out = b.evaluate(req, provider)?;
    let ms = start.elapsed().as_secs_f64() * 1000.0;
    if out.cache_hit != expected_hit || out.scene != req.authority() {
        return Err("CUDA04D CACHE OR SCENE AUTHORITY DRIFT".into());
    }
    compare(&cpu(req)?, &out.samples)?;
    Ok(ms)
}
fn case(
    device: Device,
    req: &Request,
    lib: Option<&Path>,
    hash: Option<[u8; 32]>,
) -> Result<(), String> {
    let mut b = ResidentBridge::new_managed(device, hash)?;
    if b.evaluate(req, lib).is_ok() || b.health().state != HealthState::Unverified {
        return Err("CUDA04D UNVERIFIED GATE LEAK".into());
    }
    let g1 = b.report_device_recovered()?;
    let cold = check(&mut b, req, lib, false)?;
    let cached = check(&mut b, req, lib, true)?;
    if b.resident_bytes() == 0 {
        return Err("CUDA04D NO RESIDENT BYTES".into());
    }
    let g2 = b.report_device_lost()?;
    if b.resident_bytes() != 0
        || b.evaluate(req, lib).is_ok()
        || b.health().state != HealthState::Lost
    {
        return Err("CUDA04D LOSS DID NOT REVOKE".into());
    }
    let g3 = b.report_device_recovered()?;
    let restored = check(&mut b, req, lib, false)?;
    check(&mut b, req, lib, true)?;
    if !(g1 < g2 && g2 < g3) {
        return Err("CUDA04D GENERATION NONMONOTONIC".into());
    }
    println!("CUDA04D_HEALTH_PASS device={device:?} generations={g1},{g2},{g3} cold_ms={cold:.5} hit_ms={cached:.5} restored_ms={restored:.5} parity=PASS loss_blocked=PASS");
    Ok(())
}
fn integrity(req: &Request, source: &Path) -> Result<(), String> {
    let bytes = fs::read(source).map_err(|e| e.to_string())?;
    let tmp = env::temp_dir().join(format!("blobin-cuda04d-integrity-{}", std::process::id()));
    fs::create_dir_all(&tmp).map_err(|e| e.to_string())?;
    let active = tmp.join("active.so");
    fs::write(&active, &bytes).map_err(|e| e.to_string())?;
    let approved = inspect(&active)?.fingerprint.sha256;
    let mut b = ResidentBridge::new_managed(Device::Cuda, Some(approved))?;
    b.report_device_recovered()?;
    check(&mut b, req, Some(&active), false)?;
    check(&mut b, req, Some(&active), true)?;
    let mut corrupted = bytes.clone();
    corrupted[0] ^= 0xff;
    let replacement = tmp.join("replacement.so");
    fs::write(&replacement, &corrupted).map_err(|e| e.to_string())?;
    fs::rename(&replacement, &active).map_err(|e| e.to_string())?;
    if b.evaluate(req, Some(&active)).is_ok() || b.resident_bytes() != 0 || b.provider().is_some() {
        return Err("CUDA04D SAME PATH TAMPER ACCEPTED".into());
    }
    let repaired = tmp.join("repaired.so");
    fs::write(&repaired, &bytes).map_err(|e| e.to_string())?;
    fs::rename(&repaired, &active).map_err(|e| e.to_string())?;
    check(&mut b, req, Some(&active), false)?;
    check(&mut b, req, Some(&active), true)?;
    b.invalidate();
    fs::remove_dir_all(&tmp).map_err(|e| e.to_string())?;
    println!("CUDA04D_INTEGRITY_PASS same_path_replacement=REJECTED original_hash=RESTORED stale_cache=REJECTED parity=PASS");
    Ok(())
}
fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let lib = match (args.next().as_deref(), args.next()) {
        (Some("--cuda-lib"), Some(path)) => PathBuf::from(path),
        _ => return Err("CUDA04D USAGE: --cuda-lib /absolute/file.so".into()),
    };
    if args.next().is_some() {
        return Err("CUDA04D UNKNOWN ARG".into());
    }
    let sha = inspect(&lib)?.fingerprint.sha256;
    let req = Request::from_f2(Scene::default(), generate_scene(16))?;
    case(Device::Cuda, &req, Some(&lib), Some(sha))?;
    case(Device::Vulkan, &req, None, None)?;
    integrity(&req, &lib)?;
    println!("CUDA04D_ALL_PASS managed_gpu=2 simulated_loss=2 native_same_path_tamper=1");
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("CUDA04D_ERROR {e}");
        std::process::exit(1);
    }
}
