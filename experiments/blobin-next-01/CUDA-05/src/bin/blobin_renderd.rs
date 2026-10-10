//! BLOB-FRAME-v1 preview daemon: local seqpacket -> CUDA-05F gate -> new frame worker -> sealed memfd.
//! Not production-authorized nor a proof of independent UID isolation.
use blobin_cuda04_bridge::{geometry_id, provider, Device};
use blobin_cuda05_ingress::{
    frame_protocol::{ClientMessage, MAX_FRAME, VERSION},
    frame_transport::{self, make_sealed_frame},
    isolation::{Injection, IsolatedSupervisor, Policy},
};
use serde_json::{json, Value};
use std::{
    env,
    io::{Read, Write},
    os::fd::AsRawFd,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::{Duration, Instant},
};

static STOP: AtomicBool = AtomicBool::new(false);
extern "C" fn on_shutdown(_signal: libc::c_int) {
    STOP.store(true, Ordering::Relaxed);
}
fn install_shutdown_handlers() -> Result<(), String> {
    let mut signal_action: libc::sigaction = unsafe { std::mem::zeroed() };
    signal_action.sa_sigaction = on_shutdown as *const () as usize;
    signal_action.sa_flags = 0; // accept4/recv are interrupted, not silently restarted
    unsafe {
        libc::sigemptyset(&mut signal_action.sa_mask);
    }
    for signal in [libc::SIGINT, libc::SIGTERM] {
        if unsafe { libc::sigaction(signal, &signal_action, std::ptr::null_mut()) } < 0 {
            return Err("FRAME_SIGNAL_INSTALL_FAILED".into());
        }
    }
    Ok(())
}

struct Config {
    socket: PathBuf,
    frame_worker: PathBuf,
    frame_sha: [u8; 32],
    gate_worker: PathBuf,
    gate_sha: [u8; 32],
    library: Option<PathBuf>,
    library_sha: Option<[u8; 32]>,
    timeout: Duration,
    max_requests: u64,
}
fn sha_decode(value: &str) -> Result<[u8; 32], String> {
    if value.len() != 64 || !value.is_ascii() {
        return Err("FRAME_EXPECTED_SHA256_HEX64".into());
    }
    let mut out = [0u8; 32];
    for (i, piece) in value.as_bytes().chunks_exact(2).enumerate() {
        out[i] = u8::from_str_radix(
            std::str::from_utf8(piece).map_err(|_| "FRAME_BAD_UTF8")?,
            16,
        )
        .map_err(|_| "FRAME_BAD_SHA_HEX")?;
    }
    Ok(out)
}
fn parse_args() -> Result<Config, String> {
    let mut socket = None;
    let mut frame_worker = None;
    let mut frame_sha = None;
    let mut gate_worker = None;
    let mut gate_sha = None;
    let mut library = None;
    let mut library_sha = None;
    let mut timeout_ms = 15_000u64;
    let mut max_requests = 0u64;
    let mut args = env::args().skip(1);
    while let Some(k) = args.next() {
        let v = args.next().ok_or("FRAME_CLI_VALUE_REQUIRED")?;
        match k.as_str() {
            "--socket" => socket = Some(PathBuf::from(v)),
            "--frame-worker" => frame_worker = Some(PathBuf::from(v)),
            "--frame-sha256" => frame_sha = Some(sha_decode(&v)?),
            "--gate-worker" => gate_worker = Some(PathBuf::from(v)),
            "--gate-sha256" => gate_sha = Some(sha_decode(&v)?),
            "--cuda-lib" => library = Some(PathBuf::from(v)),
            "--cuda-sha256" => library_sha = Some(sha_decode(&v)?),
            "--timeout-ms" => timeout_ms = v.parse().map_err(|_| "FRAME_TIMEOUT_PARSE")?,
            "--max-requests" => {
                max_requests = v.parse().map_err(|_| "FRAME_REQUEST_COUNT_PARSE")?
            }
            _ => return Err("FRAME_UNKNOWN_CLI_FLAG".into()),
        }
    }
    if !(500..=20_000).contains(&timeout_ms) {
        return Err("FRAME_TIMEOUT_OUT_OF_RANGE".into());
    }
    let frame_worker = frame_worker.ok_or("FRAME_WORKER_REQUIRED")?;
    let gate_worker = gate_worker.ok_or("FRAME_GATE_WORKER_REQUIRED")?;
    if !frame_worker.is_absolute() || !gate_worker.is_absolute() {
        return Err("FRAME_WORKERS_MUST_BE_ABSOLUTE".into());
    }
    if library.is_some() != library_sha.is_some()
        || library.as_ref().is_some_and(|p| !p.is_absolute())
    {
        return Err("FRAME_CUDA_LIB_AND_APPROVAL_REQUIRED_TOGETHER".into());
    }
    Ok(Config {
        socket: socket.ok_or("FRAME_SOCKET_REQUIRED")?,
        frame_worker,
        frame_sha: frame_sha.ok_or("FRAME_WORKER_SHA_REQUIRED")?,
        gate_worker,
        gate_sha: gate_sha.ok_or("FRAME_GATE_SHA_REQUIRED")?,
        library,
        library_sha,
        timeout: Duration::from_millis(timeout_ms),
        max_requests,
    })
}
fn check_binary(path: &Path, sha: [u8; 32]) -> Result<(), String> {
    let check = provider::inspect(path)?;
    if check.fingerprint.sha256 != sha {
        return Err("FRAME_APPROVED_EXECUTABLE_CHANGED".into());
    }
    Ok(())
}
struct State {
    cfg: Config,
    cuda_gate: Option<IsolatedSupervisor>,
    vulkan_gate: Option<IsolatedSupervisor>,
    frames: u64,
}
impl State {
    fn gate(&mut self, device: Device) -> Result<u64, String> {
        if device == Device::Cpu {
            return Ok(0);
        }
        let entry = match device {
            Device::Cuda => &mut self.cuda_gate,
            Device::Vulkan => &mut self.vulkan_gate,
            Device::Cpu => unreachable!(),
        };
        if entry.is_none() {
            check_binary(&self.cfg.gate_worker, self.cfg.gate_sha)?;
            let p = Policy::new(
                device,
                self.cfg.gate_worker.clone(),
                self.cfg.gate_sha,
                if device == Device::Cuda {
                    self.cfg.library.clone()
                } else {
                    None
                },
                if device == Device::Cuda {
                    self.cfg.library_sha
                } else {
                    None
                },
                self.cfg.timeout,
            )?;
            *entry = Some(IsolatedSupervisor::new(p));
        }
        let gate = entry.as_mut().expect("created");
        let result = if gate.generation().is_none() {
            gate.acquire(Injection::None)
        } else {
            gate.render(Injection::None)
        };
        if result.is_err() {
            *entry = None;
        }
        result
    }
    fn frame(&mut self, msg: &ClientMessage) -> Result<(Value, Vec<u8>), String> {
        let ClientMessage::RenderRequest {
            request_id,
            scene_revision,
            width_px,
            height_px,
            backend_requested,
            view_mode,
            ..
        } = msg
        else {
            return Err("FRAME_RENDER_MESSAGE_REQUIRED".into());
        };
        let req = msg.make_request()?;
        let device = match backend_requested.as_str() {
            "cpu" => Device::Cpu,
            "cuda" => Device::Cuda,
            "vulkan" => Device::Vulkan,
            _ => return Err("INVALID_BACKEND".into()),
        };
        check_binary(&self.cfg.frame_worker, self.cfg.frame_sha)?;
        if device == Device::Cuda {
            let p = self
                .cfg
                .library
                .as_deref()
                .ok_or("FRAME_CUDA_PROVIDER_NOT_CONFIGURED")?;
            check_binary(
                p,
                self.cfg
                    .library_sha
                    .ok_or("FRAME_CUDA_PROVIDER_SHA_NOT_CONFIGURED")?,
            )?;
        }
        let request_started = Instant::now();
        let gate_started = Instant::now();
        let generation = self.gate(device)?;
        let gate_ms = gate_started.elapsed().as_secs_f64() * 1000.0;
        let pixels = (*width_px as usize)
            .checked_mul(*height_px as usize)
            .ok_or("FRAME_SIZE_OVERFLOW")?;
        let expected = pixels.checked_mul(4).ok_or("FRAME_SIZE_OVERFLOW")?;
        if expected > MAX_FRAME {
            return Err("FRAME_BUFFER_TOO_LARGE".into());
        }
        let mut args = vec!["--backend".to_string(), backend_requested.clone()];
        if device == Device::Cuda {
            args.extend([
                "--cuda-lib".into(),
                self.cfg
                    .library
                    .as_ref()
                    .expect("verified")
                    .to_string_lossy()
                    .to_string(),
                "--sha256".into(),
                self.cfg
                    .library_sha
                    .expect("verified")
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect::<String>(),
            ]);
        }
        let worker_started = Instant::now();
        let mut child = Command::new(&self.cfg.frame_worker)
            .args(&args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|_| "FRAME_SPAWN_FAILED")?;
        let out = child.stdout.take().ok_or("FRAME_WORKER_STDOUT_MISSING")?;
        let reader = thread::spawn(move || -> Result<Vec<u8>, String> {
            let mut bytes = Vec::new();
            out.take((MAX_FRAME + 1) as u64)
                .read_to_end(&mut bytes)
                .map_err(|_| "FRAME_WORKER_READ_FAILED")?;
            Ok(bytes)
        });
        let scene = serde_json::to_vec(msg).map_err(|_| "FRAME_SERIALIZATION_FAILED")?;
        if scene.len() > blobin_cuda05_ingress::frame_protocol::MAX_CONTROL {
            return Err("FRAME_SCENE_TOO_LARGE".into());
        }
        // The request JSON is bounded; writing to the pipe cannot fill an ordinary Linux pipe.
        let send = child
            .stdin
            .take()
            .ok_or("FRAME_CHILD_STDIN_MISSING")?
            .write_all(&scene)
            .map_err(|_| "FRAME_CHILD_STDIN_FAILED");
        if send.is_err() {
            let _ = child.kill();
            let _ = child.wait();
            return Err("FRAME_CHILD_STDIN_FAILED".into());
        }
        let start = Instant::now();
        let status = loop {
            match child.try_wait() {
                Ok(Some(s)) => break s,
                Ok(None) if start.elapsed() > self.cfg.timeout => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err("FRAME_GPU_WORKER_TIMEOUT_REVOKED".into());
                }
                Ok(None) => thread::sleep(Duration::from_millis(10)),
                Err(_) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err("FRAME_GPU_WORKER_WAIT_FAILED".into());
                }
            }
        };
        let rgba = reader.join().map_err(|_| "FRAME_READER_PANICKED")??;
        if !status.success() {
            return Err("FRAME_WORKER_REPORTED_FAILURE".into());
        }
        if rgba.len() != expected {
            return Err("FRAME_OUTPUT_BYTE_COUNT_MISMATCH".into());
        }
        let worker_ms = worker_started.elapsed().as_secs_f64() * 1000.0;
        eprintln!("BLOB_FRAME_TIMING backend={backend_requested} mode={view_mode} width={width_px} height={height_px} gate_ms={gate_ms:.3} worker_ms={worker_ms:.3} total_ms={:.3} sequence={} descriptor=sealed_worker_per_frame",request_started.elapsed().as_secs_f64()*1000.0,self.frames+1);
        self.frames = self.frames.checked_add(1).ok_or("FRAME_COUNTER_OVERFLOW")?;
        let hash = geometry_id(&req)?
            .0
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>();
        let qualification = if device == Device::Cpu {
            "CPU_LOCAL_PREVIEW_NOT_GPU_QUALIFIED"
        } else {
            "CUDA05F_GATE_PASS"
        };
        Ok((
            json!({
                "version":VERSION,"type":"FRAME_READY","request_id":request_id,
                "scene_revision":scene_revision,"width_px":width_px,"height_px":height_px,
                "stride_bytes":width_px*4,"format":"RGBA8_UNORM_OPAQUE","buffer_bytes":expected,
                "backend_requested":backend_requested,"backend_actual":backend_requested,
                "view_mode":view_mode,"generation":generation,"qualification":qualification,
                "cache_hit":false,"geometry_sha256":hash,"frame_sequence":self.frames
            }),
            rgba,
        ))
    }
}
fn process(client: std::os::fd::RawFd, state: &mut State) -> Result<(), String> {
    loop {
        let received = match frame_transport::recv_packet(client) {
            Ok(Some(p)) => p,
            Ok(None) => return Ok(()),
            Err(_) => return Err("FRAME_CLIENT_PACKET_INVALID".into()),
        };
        let msg = match ClientMessage::parse(&received) {
            Ok(m) => m,
            Err(_) => {
                let response = json!({"version":VERSION,"type":"RENDER_ERROR","request_id":0,"scene_revision":0,"reason":"INVALID_CONTROL_MESSAGE"});
                let _ = frame_transport::send_packet(client, response.to_string().as_bytes(), None);
                continue;
            }
        };
        let result = match &msg {
            ClientMessage::Hello { .. } => Ok((
                json!({"version":VERSION,"type":"HELLO","server":"blobin-cuda05f-render-bridge","max_control_bytes":16384}),
                None,
            )),
            ClientMessage::Capabilities { .. } => Ok((
                json!({"version":VERSION,"type":"CAPABILITIES",
                "backends_implemented":["cpu","cuda","vulkan"],"cuda_configured":state.cfg.library.is_some(),
                "view_modes":["lit","normals","distance","coverage"],
                "max_width_px":1024,"max_height_px":1024,"max_pixels":1_048_576,
                "device_healthy_now":"UNVERIFIED_UNTIL_RENDER"}),
                None,
            )),
            ClientMessage::Ping { .. } => Ok((json!({"version":VERSION,"type":"PONG"}), None)),
            ClientMessage::RenderRequest { .. } => state.frame(&msg).map(|(j, b)| (j, Some(b))),
        };
        match result {
            Ok((reply, Some(bytes))) => {
                let fd = make_sealed_frame(&bytes)?;
                let body = reply.to_string();
                frame_transport::send_packet(client, body.as_bytes(), Some(fd.as_raw_fd()))?;
            }
            Ok((reply, None)) => {
                frame_transport::send_packet(client, reply.to_string().as_bytes(), None)?
            }
            Err(reason) => {
                eprintln!("FRAME_RENDER_REJECTED class={reason}");
                // The client is not entitled to operational file paths or provider diagnostics.
                let (request_id, scene_revision) = match msg {
                    ClientMessage::RenderRequest {
                        request_id,
                        scene_revision,
                        ..
                    } => (request_id, scene_revision),
                    _ => (0, 0),
                };
                let code = if reason.contains("TIMEOUT") {
                    "WORKER_TIMEOUT"
                } else {
                    "RENDER_FAILED"
                };
                let reply = json!({"version":VERSION,"type":"RENDER_ERROR","request_id":request_id,
                    "scene_revision":scene_revision,"reason":code});
                frame_transport::send_packet(client, reply.to_string().as_bytes(), None)?;
            }
        }
    }
}
fn run() -> Result<(), String> {
    let cfg = parse_args()?;
    check_binary(&cfg.frame_worker, cfg.frame_sha)?;
    check_binary(&cfg.gate_worker, cfg.gate_sha)?;
    if let (Some(path), Some(sha)) = (cfg.library.as_deref(), cfg.library_sha) {
        check_binary(path, sha)?;
    }
    let path = cfg.socket.clone();
    let listener = frame_transport::listen(&path)?;
    install_shutdown_handlers()?;
    println!(
        "BLOB_FRAME_LISTENING protocol={VERSION} socket={} qualification=PREVIEW_ONLY",
        path.display()
    );
    let mut state = State {
        cfg,
        cuda_gate: None,
        vulkan_gate: None,
        frames: 0,
    };
    let mut n = 0u64;
    loop {
        if STOP.load(Ordering::Relaxed) {
            break;
        }
        if state.cfg.max_requests != 0 && n >= state.cfg.max_requests {
            break;
        }
        let client = match frame_transport::accept(&listener) {
            Ok(c) => c,
            Err(e) => {
                if STOP.load(Ordering::Relaxed) {
                    break;
                }
                eprintln!("FRAME_PEER_REJECTED {e}");
                continue;
            }
        };
        n += 1;
        if let Err(error) = process(client.as_raw_fd(), &mut state) {
            eprintln!("FRAME_CLIENT_TERMINATED reason={error}");
        }
    }
    drop(listener);
    std::fs::remove_file(&path).map_err(|e| e.to_string())?;
    println!("BLOB_FRAME_EXIT clean=PASS frames={}", state.frames);
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("BLOB_FRAME_FATAL {e}");
        std::process::exit(2);
    }
}
