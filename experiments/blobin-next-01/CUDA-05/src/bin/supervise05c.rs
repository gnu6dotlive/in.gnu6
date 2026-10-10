//! Finite CUDA-05C socket supervisor: GPU operations run only in disposable children.
use blobin_cuda04_bridge::Device;
use blobin_cuda05_ingress::{
    isolation::{Injection, IsolatedSupervisor, Policy},
    socket,
};
use std::{
    env, fs,
    io::{Read, Write},
    os::unix::{fs::PermissionsExt, net::UnixListener},
    path::PathBuf,
    time::Duration,
};
struct Args {
    socket: PathBuf,
    worker: PathBuf,
    worker_sha256: [u8; 32],
    device: Device,
    lib: Option<PathBuf>,
    hash: Option<[u8; 32]>,
    uid: u32,
    limit: usize,
    timeout_ms: u64,
    injected_index: Option<usize>,
    fault: Injection,
}
fn decode_hash(s: &str) -> Result<[u8; 32], String> {
    if s.len() != 64 {
        return Err("CUDA05C HASH HEX64 REQUIRED".into());
    }
    let mut bytes = [0u8; 32];
    for (i, pair) in s.as_bytes().chunks_exact(2).enumerate() {
        bytes[i] = u8::from_str_radix(std::str::from_utf8(pair).map_err(|e| e.to_string())?, 16)
            .map_err(|_| "CUDA05C BAD HASH")?;
    }
    Ok(bytes)
}
fn parse() -> Result<Args, String> {
    let mut args = env::args().skip(1);
    let (
        mut socket,
        mut worker,
        mut worker_sha256,
        mut device,
        mut lib,
        mut hash,
        mut uid,
        mut limit,
        mut timeout_ms,
        mut injected_index,
        mut fault,
    ) = (
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Injection::None,
    );
    while let Some(name) = args.next() {
        let val = args.next().ok_or("CUDA05C CLI VALUE REQUIRED")?;
        match name.as_str() {
            "--socket" => socket = Some(PathBuf::from(val)),
            "--worker" => worker = Some(PathBuf::from(val)),
            "--worker-sha256" => worker_sha256 = Some(decode_hash(&val)?),
            "--backend" => {
                device = Some(match val.as_str() {
                    "cuda" => Device::Cuda,
                    "vulkan" => Device::Vulkan,
                    _ => return Err("CUDA05C UNKNOWN BACKEND".into()),
                })
            }
            "--cuda-lib" => lib = Some(PathBuf::from(val)),
            "--sha256" => hash = Some(decode_hash(&val)?),
            "--allowed-uid" => uid = Some(val.parse().map_err(|_| "CUDA05C BAD UID")?),
            "--max-requests" => limit = Some(val.parse().map_err(|_| "CUDA05C BAD LIMIT")?),
            "--timeout-ms" => timeout_ms = Some(val.parse().map_err(|_| "CUDA05C BAD TIMEOUT")?),
            "--fault-index" => injected_index = Some(val.parse().map_err(|_| "CUDA05C BAD INDEX")?),
            "--fault" => {
                fault = match val.as_str() {
                    "crash" => Injection::Crash,
                    "hang" => Injection::Hang,
                    _ => return Err("CUDA05C BAD INJECTION".into()),
                }
            }
            _ => return Err(format!("CUDA05C UNKNOWN FLAG {name}")),
        }
    }
    let limit = limit.ok_or("CUDA05C LIMIT REQUIRED")?;
    if !(1..=24).contains(&limit) {
        return Err("CUDA05C LIMIT OUT OF RANGE".into());
    }
    if let Some(i) = injected_index {
        if i >= limit {
            return Err("CUDA05C BAD INDEX".into());
        }
    }
    if injected_index.is_some() != (fault != Injection::None) {
        return Err("CUDA05C FAULT REQUIRES INDEX AND MODE".into());
    }
    Ok(Args {
        socket: socket.ok_or("CUDA05C SOCKET REQUIRED")?,
        worker: worker.ok_or("CUDA05C WORKER REQUIRED")?,
        worker_sha256: worker_sha256.ok_or("CUDA05C WORKER APPROVAL REQUIRED")?,
        device: device.ok_or("CUDA05C BACKEND REQUIRED")?,
        lib,
        hash,
        uid: uid.ok_or("CUDA05C UID REQUIRED")?,
        limit,
        timeout_ms: timeout_ms.ok_or("CUDA05C TIMEOUT REQUIRED")?,
        injected_index,
        fault,
    })
}
fn execute(
    s: &mut IsolatedSupervisor,
    command: &str,
    injected: Injection,
) -> Result<String, String> {
    match command {
        "STATUS" => Ok(format!(
            "STATE generation={:?} launches={}",
            s.generation(),
            s.launches()
        )),
        "ACQUIRE" => Ok(format!("READY generation={}", s.acquire(injected)?)),
        "RENDER" => Ok(format!(
            "RENDER_PASS generation={} worker=FRESH parity=PASS",
            s.render(injected)?
        )),
        "REVOKE" => {
            s.revoke();
            Ok("REVOKED".into())
        }
        _ => Err("CUDA05C UNKNOWN COMMAND".into()),
    }
}
fn run() -> Result<(), String> {
    let a = parse()?;
    socket::validate_socket_destination(&a.socket)?;
    let policy = Policy::new(
        a.device,
        a.worker,
        a.worker_sha256,
        a.lib,
        a.hash,
        Duration::from_millis(a.timeout_ms),
    )?;
    let mut supervisor = IsolatedSupervisor::new(policy);
    let listener = UnixListener::bind(&a.socket).map_err(|e| e.to_string())?;
    fs::set_permissions(&a.socket, fs::Permissions::from_mode(0o600)).map_err(|e| e.to_string())?;
    println!(
        "CUDA05C_LISTENING backend={:?} limits={} timeout_ms={}",
        a.device, a.limit, a.timeout_ms
    );
    for i in 0..a.limit {
        let (mut stream, _) = listener.accept().map_err(|e| e.to_string())?;
        stream
            .set_read_timeout(Some(Duration::from_secs(3)))
            .map_err(|e| e.to_string())?;
        let result = match socket::authorize(&stream, a.uid) {
            Ok(peer) => {
                let mut buf = [0u8; 64];
                let n = stream.read(&mut buf).map_err(|e| e.to_string())?;
                let mode = if a.injected_index == Some(i) {
                    a.fault
                } else {
                    Injection::None
                };
                match std::str::from_utf8(&buf[..n]) {
                    Ok(line) if line.ends_with('\n') && line.len() <= 32 => {
                        execute(&mut supervisor, line.trim_end(), mode)
                    }
                    _ => Err("CUDA05C MALFORMED COMMAND".into()),
                }
                .map(|v| format!("{v} peer_uid={}", peer.uid))
            }
            Err(_) => Err("CUDA05C PEER UNAUTHORIZED".into()),
        };
        let response = match result {
            Ok(v) => v,
            Err(e) => format!("DENIED reason={}", e.replace(" ", "_")),
        };
        writeln!(stream, "{response}").map_err(|e| e.to_string())?;
        println!("CUDA05C_EVENT index={i} {response}");
    }
    supervisor.revoke();
    drop(listener);
    fs::remove_file(&a.socket).map_err(|e| e.to_string())?;
    println!("CUDA05C_EXIT clean=PASS");
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("CUDA05C_ERROR {e}");
        std::process::exit(1)
    }
}
