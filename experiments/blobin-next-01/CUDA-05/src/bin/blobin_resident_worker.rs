//! BLOB-02C: disposable, bounded *resident* GPU child. NO supervisor authority here.
//! Local research protocol: stdin (u32_le len, JSON), stdout (u32_le len, RGBA).
//! Only a trusted parent may launch this executable after approving its bytes.
use blobin_cuda04_bridge::{provider, resident::ResidentBridge, Device};
use blobin_cuda05_ingress::frame_protocol::{colorize, ClientMessage, MAX_CONTROL, MAX_FRAME};
use std::{
    env,
    io::{self, Read, Write},
    path::PathBuf,
    time::{Duration, Instant},
};
fn sha_hex(src: &str) -> Result<[u8; 32], String> {
    if src.len() != 64 || !src.is_ascii() {
        return Err("INVALID_APPROVED_SHA".into());
    }
    let mut b = [0u8; 32];
    for (i, c) in src.as_bytes().chunks_exact(2).enumerate() {
        b[i] = u8::from_str_radix(std::str::from_utf8(c).map_err(|_| "INVALID_HEX_UTF8")?, 16)
            .map_err(|_| "INVALID_HEX")?;
    }
    Ok(b)
}
fn main() {
    if let Err(e) = run() {
        eprintln!("RESIDENT_WORKER_FAIL {e}");
        std::process::exit(2);
    }
}
fn run() -> Result<(), String> {
    let mut a = env::args().skip(1);
    let backend = a.next().ok_or("REQUIRE_BACKEND")?;
    let dev = match backend.as_str() {
        "cuda" => Device::Cuda,
        "vulkan" => Device::Vulkan,
        _ => return Err("GPU_ONLY".into()),
    };
    let lib_arg = a.next().ok_or("REQUIRE_LIBRARY_OR_DASH")?;
    let sha_arg = a.next().ok_or("REQUIRE_SHA_OR_DASH")?;
    let max_frames: usize = a
        .next()
        .ok_or("REQUIRE_FRAME_BUDGET")?
        .parse()
        .map_err(|_| "INVALID_FRAME_BUDGET")?;
    let ttl_ms: u64 = a
        .next()
        .ok_or("REQUIRE_SESSION_TTL_MS")?
        .parse()
        .map_err(|_| "INVALID_TTL")?;
    if a.next().is_some() || !(1..=240).contains(&max_frames) || !(1000..=120000).contains(&ttl_ms)
    {
        return Err("RESIDENT_BUDGET_BOUNDS".into());
    }
    let lib = if dev == Device::Cuda {
        let p = PathBuf::from(&lib_arg);
        if !p.is_absolute() {
            return Err("ABSOLUTE_CUDA_LIBRARY_REQUIRED".into());
        }
        Some(p)
    } else {
        None
    };
    if dev == Device::Vulkan && (lib_arg != "-" || sha_arg != "-") {
        return Err("VULKAN_REQUIRES_NO_CUDA_LIBRARY".into());
    }
    let approved = if dev == Device::Cuda {
        Some(sha_hex(&sha_arg)?)
    } else {
        None
    };
    if let (Some(p), Some(hash)) = (lib.as_ref(), approved) {
        if provider::inspect(p)?.fingerprint.sha256 != hash {
            return Err("CUDA_PROVIDER_SHA_MISMATCH".into());
        }
    }
    let mut bridge = ResidentBridge::new_managed(dev, approved)?;
    bridge.report_device_recovered()?;
    let mut stdin = io::stdin().lock();
    let mut stdout = io::stdout().lock();
    let started = Instant::now();
    for seq in 0..max_frames {
        if started.elapsed() > Duration::from_millis(ttl_ms) {
            return Err("SESSION_TTL_EXPIRED_REVOKED".into());
        }
        let mut prefix = [0u8; 4];
        match stdin.read(&mut prefix[..1]) {
            Ok(0) => {
                eprintln!("RESIDENT_WORKER_EOF frames={seq}");
                return Ok(());
            }
            Ok(1) => {}
            _ => return Err("REQUEST_PREFIX_READ_FAILED".into()),
        }
        stdin
            .read_exact(&mut prefix[1..])
            .map_err(|_| "TRUNCATED_REQUEST_PREFIX")?;
        let bytes = u32::from_le_bytes(prefix) as usize;
        if bytes == 0 || bytes > MAX_CONTROL {
            return Err("REQUEST_LENGTH_INVALID".into());
        }
        let mut payload = vec![0u8; bytes];
        stdin
            .read_exact(&mut payload)
            .map_err(|_| "TRUNCATED_JSON_PAYLOAD")?;
        if started.elapsed() > Duration::from_millis(ttl_ms) {
            return Err("SESSION_TTL_EXPIRED_REVOKED".into());
        }
        let message = ClientMessage::parse(&payload)?;
        let mode = match &message {
            ClientMessage::RenderRequest {
                backend_requested,
                view_mode,
                ..
            } => {
                if backend_requested != &backend {
                    return Err("BACKEND_MISMATCH".into());
                }
                view_mode.as_str()
            }
            _ => return Err("FRAME_MESSAGE_REQUIRED".into()),
        };
        let request = message.make_request()?;
        let output = bridge.evaluate_observed(&request, lib.as_deref())?;
        if output.output.selected != dev {
            return Err("RESIDENT_UNEXPECTED_DEVICE".into());
        }
        // CUDA-04 reuses validated SDF samples when the geometry is unchanged.
        // A cache hit is not a device failure: still colorize for the requested mode,
        // and send provenance to the parent without claiming fresh GPU execution.
        let cached = output.output.cache_hit;
        let pixels = request.dimensions().0 as usize * request.dimensions().1 as usize;
        if output.output.samples.len() != pixels {
            return Err("SAMPLE_LENGTH_WRONG".into());
        }
        let rgba = colorize(&output.output.samples, mode)?;
        if rgba.len() != pixels * 4 || rgba.len() > MAX_FRAME {
            return Err("RGBA_BOUNDS".into());
        }
        let len = (rgba.len() as u32).to_le_bytes();
        stdout
            .write_all(&len)
            .map_err(|_| "RESPONSE_PREFIX_FAILED")?;
        stdout
            .write_all(&[u8::from(cached)])
            .map_err(|_| "RESPONSE_CACHE_FLAG_FAILED")?;
        stdout
            .write_all(&rgba)
            .map_err(|_| "RESPONSE_PAYLOAD_FAILED")?;
        stdout.flush().map_err(|_| "RESPONSE_FLUSH_FAILED")?;
        if seq == 0 {
            eprintln!(
                "RESIDENT_WORKER_WARM backend={backend} bytes={}",
                rgba.len()
            );
        }
    }
    eprintln!("RESIDENT_WORKER_BUDGET_REVOKED max_frames={max_frames}");
    Ok(())
}
