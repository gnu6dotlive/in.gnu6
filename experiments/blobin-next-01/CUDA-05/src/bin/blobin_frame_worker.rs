//! Separate disposable compute process. Requires a parsed bounded typed frame request.
//! Does not receive client paths, shell commands, or runtime access-control decisions.
use blobin_cuda02_provider as geometry;
use blobin_cuda04_bridge::{provider, resident::ResidentBridge, Device};
use blobin_cuda05_ingress::frame_protocol::{colorize, ClientMessage, MAX_CONTROL};
use std::{
    env,
    io::{Read, Write},
    path::PathBuf,
};

fn decode_hex(src: &str) -> Result<[u8; 32], String> {
    if src.len() != 64 || !src.is_ascii() {
        return Err("FRAME_WORKER_BAD_APPROVED_SHA".into());
    }
    let mut out = [0u8; 32];
    for (i, piece) in src.as_bytes().chunks_exact(2).enumerate() {
        out[i] = u8::from_str_radix(std::str::from_utf8(piece).map_err(|_| "SHA_UTF8")?, 16)
            .map_err(|_| "SHA_INVALID_HEX")?;
    }
    Ok(out)
}
fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let mut backend = None;
    let mut lib = None;
    let mut approved = None;
    while let Some(arg) = args.next() {
        let val = args.next().ok_or("FRAME_WORKER_MISSING_VALUE")?;
        match arg.as_str() {
            "--backend" => {
                backend = Some(match val.as_str() {
                    "cpu" => Device::Cpu,
                    "cuda" => Device::Cuda,
                    "vulkan" => Device::Vulkan,
                    _ => return Err("FRAME_WORKER_INVALID_BACKEND".into()),
                })
            }
            "--cuda-lib" => lib = Some(PathBuf::from(val)),
            "--sha256" => approved = Some(decode_hex(&val)?),
            _ => return Err("FRAME_WORKER_UNKNOWN_FLAG".into()),
        }
    }
    let backend = backend.ok_or("FRAME_WORKER_BACKEND_REQUIRED")?;
    if backend == Device::Cuda && (lib.is_none() || approved.is_none()) {
        return Err("FRAME_WORKER_CUDA_PIN_REQUIRED".into());
    }
    let mut buf = Vec::new();
    std::io::stdin()
        .take((MAX_CONTROL + 1) as u64)
        .read_to_end(&mut buf)
        .map_err(|e| format!("FRAME_WORKER_INPUT: {e}"))?;
    let msg = ClientMessage::parse(&buf)?;
    let mode = match &msg {
        ClientMessage::RenderRequest {
            backend_requested,
            view_mode,
            ..
        } => {
            if backend_requested
                != match backend {
                    Device::Cpu => "cpu",
                    Device::Cuda => "cuda",
                    Device::Vulkan => "vulkan",
                }
            {
                return Err("FRAME_WORKER_BACKEND_MISMATCH".into());
            }
            view_mode.as_str()
        }
        _ => return Err("FRAME_WORKER_RENDER_ONLY".into()),
    };
    let req = msg.make_request()?;
    let samples = match backend {
        Device::Cpu => geometry::cpu(&req)?,
        Device::Cuda | Device::Vulkan => {
            let pinned = if backend == Device::Cuda {
                let path = lib.as_deref().ok_or("CUDA_LIBRARY_REQUIRED")?;
                let check = provider::inspect(path)?;
                if Some(check.fingerprint.sha256) != approved {
                    return Err("FRAME_WORKER_CUDA_SHA_CHANGED".into());
                }
                approved
            } else {
                None
            };
            let mut bridge = ResidentBridge::new_managed(backend, pinned)?;
            bridge.report_device_recovered()?;
            let observed = bridge.evaluate_observed(&req, lib.as_deref())?;
            if observed.output.selected != backend || observed.output.cache_hit {
                return Err("FRAME_WORKER_UNEXPECTED_BACKEND_OR_CACHE".into());
            }
            observed.output.samples
        }
    };
    let pixels = req.dimensions().0 as usize * req.dimensions().1 as usize;
    if samples.len() != pixels {
        return Err("FRAME_WORKER_SAMPLE_COUNT".into());
    }
    let rgba = colorize(&samples, mode)?;
    if rgba.len() != pixels * 4 {
        return Err("FRAME_WORKER_RGBA_COUNT".into());
    }
    std::io::stdout()
        .write_all(&rgba)
        .map_err(|e| format!("FRAME_WORKER_OUTPUT {e}"))?;
    std::io::stdout().flush().map_err(|e| e.to_string())?;
    eprintln!("FRAME_WORKER_OK selected={backend:?} samples={pixels} mode={mode}");
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("FRAME_WORKER_ERROR {error}");
        std::process::exit(2)
    }
}
