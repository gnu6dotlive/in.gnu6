//! Finite laboratory service; Unix peer credentials and fixed GPU workload.
use blobin_cuda02_provider::{compare, cpu, generate_scene, Request};
use blobin_cuda04_bridge::Device;
use blobin_cuda05_ingress::{
    socket,
    supervisor::{Policy, Supervisor},
};
use blobin_f2_topology::Scene;
use std::{
    env, fs,
    io::{Read, Write},
    os::unix::{fs::PermissionsExt, net::UnixListener},
    path::PathBuf,
    time::Duration,
};
struct Args {
    socket: PathBuf,
    device: Device,
    path: Option<PathBuf>,
    hash: Option<[u8; 32]>,
    uid: u32,
    max: usize,
}
fn hash_hex(s: &str) -> Result<[u8; 32], String> {
    if s.len() != 64 {
        return Err("CUDA05B HASH HEX64 REQUIRED".into());
    }
    let mut hash = [0u8; 32];
    for (i, bytes) in s.as_bytes().chunks_exact(2).enumerate() {
        let pair = std::str::from_utf8(bytes).map_err(|e| e.to_string())?;
        hash[i] = u8::from_str_radix(pair, 16).map_err(|_| "CUDA05B BAD HASH")?;
    }
    Ok(hash)
}
fn parse() -> Result<Args, String> {
    let mut a = env::args().skip(1);
    let (mut sock, mut device, mut path, mut hash, mut uid, mut max) =
        (None, None, None, None, None, None);
    while let Some(flag) = a.next() {
        let v = a.next().ok_or("CUDA05B FLAG MISSING VALUE")?;
        match flag.as_str() {
            "--socket" => sock = Some(PathBuf::from(v)),
            "--backend" => {
                device = Some(match v.as_str() {
                    "cuda" => Device::Cuda,
                    "vulkan" => Device::Vulkan,
                    _ => return Err("CUDA05B BAD BACKEND".into()),
                })
            }
            "--cuda-lib" => path = Some(PathBuf::from(v)),
            "--sha256" => hash = Some(hash_hex(&v)?),
            "--allowed-uid" => uid = Some(v.parse().map_err(|_| "CUDA05B BAD UID")?),
            "--max-requests" => max = Some(v.parse().map_err(|_| "CUDA05B BAD LIMIT")?),
            _ => return Err("CUDA05B UNKNOWN FLAG".into()),
        }
    }
    let max = max.ok_or("CUDA05B LIMIT REQUIRED")?;
    if !(1..=32).contains(&max) {
        return Err("CUDA05B LIMIT OUT OF RANGE".into());
    }
    Ok(Args {
        socket: sock.ok_or("CUDA05B SOCKET REQUIRED")?,
        device: device.ok_or("CUDA05B BACKEND REQUIRED")?,
        path,
        hash,
        uid: uid.ok_or("CUDA05B UID REQUIRED")?,
        max,
    })
}
fn command(s: &mut Supervisor, cmd: &str) -> Result<String, String> {
    match cmd {
        "STATUS" => Ok(format!(
            "STATE generation={:?} challenges={}",
            s.generation(),
            s.challenge_count()
        )),
        "ACQUIRE" => Ok(format!("READY generation={}", s.acquire()?)),
        "REVOKE" => {
            s.revoke()?;
            Ok("REVOKED".into())
        }
        "RENDER" => {
            let req = Request::from_f2(Scene::default(), generate_scene(16))?;
            let frame = s.render(&req)?;
            compare(&cpu(&req)?, &frame.output.samples)?;
            Ok(format!(
                "RENDER_PASS generation={} cache_hit={} parity=PASS",
                frame.health.generation, frame.output.cache_hit
            ))
        }
        _ => Err("CUDA05B UNKNOWN COMMAND".into()),
    }
}
fn run() -> Result<(), String> {
    let a = parse()?;
    socket::validate_socket_destination(&a.socket)?;
    let mut s = Supervisor::new(Policy::new(a.device, a.path, a.hash)?)?;
    let listener = UnixListener::bind(&a.socket).map_err(|e| e.to_string())?;
    fs::set_permissions(&a.socket, fs::Permissions::from_mode(0o600)).map_err(|e| e.to_string())?;
    println!(
        "CUDA05B_LISTENING backend={:?} max_requests={} allowed_uid={}",
        a.device, a.max, a.uid
    );
    for _ in 0..a.max {
        let (mut stream, _) = listener.accept().map_err(|e| e.to_string())?;
        stream
            .set_read_timeout(Some(Duration::from_secs(4)))
            .map_err(|e| e.to_string())?;
        let result = match socket::authorize(&stream, a.uid) {
            Ok(peer) => {
                let mut buf = [0u8; 64];
                let n = stream.read(&mut buf).map_err(|e| e.to_string())?;
                match std::str::from_utf8(&buf[..n]) {
                    Ok(v) if v.ends_with('\n') && v.len() <= 32 => command(&mut s, v.trim_end()),
                    _ => Err("CUDA05B MALFORMED COMMAND".into()),
                }
                .map(|v| format!("{v} peer_uid={}", peer.uid))
            }
            Err(_) => Err("CUDA05B PEER UNAUTHORIZED".into()),
        };
        let response = match result {
            Ok(v) => v,
            Err(e) => format!("DENIED reason={}", e.replace(" ", "_")),
        };
        writeln!(stream, "{response}").map_err(|e| e.to_string())?;
        println!("CUDA05B_EVENT {response}");
    }
    let _ = s.revoke();
    drop(listener);
    fs::remove_file(&a.socket).map_err(|e| e.to_string())?;
    println!("CUDA05B_EXIT clean=PASS");
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("CUDA05B_ERROR {e}");
        std::process::exit(1)
    }
}
