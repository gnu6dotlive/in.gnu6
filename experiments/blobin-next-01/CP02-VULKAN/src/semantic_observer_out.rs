//! ORACLE-05: opt-in, bounded producer for a read-only TS semantic observer.
//! Connects to an existing 0700-parent / 0600 AF_UNIX socket; no authority
//! to mutate [in.gnu6]. Networking never runs on the Vulkan render thread.
use serde_json::Value;
use std::{
    fs,
    io::Write,
    os::{
        fd::AsRawFd,
        unix::{
            fs::{FileTypeExt, PermissionsExt},
            net::UnixStream,
        },
    },
    path::Path,
    sync::{
        atomic::{AtomicU64, Ordering},
        mpsc::{sync_channel, SyncSender, TrySendError},
        Arc,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

pub const MAX_EVENT_BYTES: usize = 2048;
pub const MAX_PENDING: usize = 16;
#[derive(Default)]
struct Counters {
    written: AtomicU64,
    failed: AtomicU64,
}
pub struct SemanticPublisher {
    sender: Option<SyncSender<Vec<u8>>>,
    worker: Option<JoinHandle<()>>,
    counters: Arc<Counters>,
}
fn same_uid(stream: &UnixStream) -> bool {
    let mut peer = libc::ucred {
        pid: 0,
        uid: 0,
        gid: 0,
    };
    let mut size = std::mem::size_of::<libc::ucred>() as libc::socklen_t;
    // SAFETY: owned socket fd and correctly sized writable stack buffers.
    let ok = unsafe {
        libc::getsockopt(
            stream.as_raw_fd(),
            libc::SOL_SOCKET,
            libc::SO_PEERCRED,
            (&raw mut peer).cast(),
            &raw mut size,
        )
    };
    ok == 0
        && size as usize == std::mem::size_of::<libc::ucred>()
        && peer.uid == unsafe { libc::geteuid() }
}
pub fn encode(value: &Value) -> Result<Vec<u8>, String> {
    let mut bytes = serde_json::to_vec(value).map_err(|e| e.to_string())?;
    if bytes.len() > MAX_EVENT_BYTES - 1 {
        return Err("ORACLE05_EVENT_TOO_LARGE".into());
    }
    bytes.push(b'\n');
    Ok(bytes)
}
impl SemanticPublisher {
    pub fn connect(socket: &Path, header: Value) -> Result<Self, String> {
        if !socket.is_absolute() {
            return Err("ORACLE05_SOCKET_NOT_ABSOLUTE".into());
        }
        let parent = socket.parent().ok_or("ORACLE05_SOCKET_NO_PARENT")?;
        let dir = fs::symlink_metadata(parent).map_err(|e| e.to_string())?;
        if !dir.is_dir() || dir.permissions().mode() & 0o077 != 0 {
            return Err("ORACLE05_SOCKET_PARENT_MUST_BE_PRIVATE".into());
        }
        let file = fs::symlink_metadata(socket).map_err(|e| e.to_string())?;
        if !file.file_type().is_socket() || file.permissions().mode() & 0o077 != 0 {
            return Err("ORACLE05_SOCKET_NOT_PRIVATE_UNIX_SOCKET".into());
        }
        let mut stream = UnixStream::connect(socket)
            .map_err(|e| format!("ORACLE05_SOCKET_CONNECT_FAILED {e}"))?;
        if !same_uid(&stream) {
            return Err("ORACLE05_PEER_UID_MISMATCH".into());
        }
        stream
            .set_write_timeout(Some(Duration::from_millis(80)))
            .map_err(|e| e.to_string())?;
        // Handshake is synchronous at launch, before renderer starts its loop.
        let first = encode(&header)?;
        stream
            .write_all(&first)
            .map_err(|e| format!("ORACLE05_HANDSHAKE_FAILED {e}"))?;
        let (tx, rx) = sync_channel::<Vec<u8>>(MAX_PENDING);
        let counters = Arc::new(Counters::default());
        let local = counters.clone();
        let thread = thread::Builder::new()
            .name("gnu6-oracle05-observer-tx".into())
            .spawn(move || {
                for packet in rx {
                    match stream.write_all(&packet) {
                        Ok(()) => {
                            local.written.fetch_add(1, Ordering::Relaxed);
                        }
                        Err(e) => {
                            eprintln!("ORACLE05_OBSERVER_DISCONNECTED {e}");
                            local.failed.fetch_add(1, Ordering::Relaxed);
                            break;
                        }
                    }
                }
            })
            .map_err(|e| format!("ORACLE05_THREAD_START_FAILED {e}"))?;
        Ok(Self {
            sender: Some(tx),
            worker: Some(thread),
            counters,
        })
    }
    pub fn emit(&self, event: &Value) -> Result<(), String> {
        if self.counters.failed.load(Ordering::Relaxed) > 0 {
            return Err("ORACLE05_OBSERVER_TRANSPORT_FAILED".into());
        }
        let bytes = encode(event)?;
        match self
            .sender
            .as_ref()
            .ok_or("ORACLE05_CLOSED")?
            .try_send(bytes)
        {
            Ok(()) => Ok(()),
            Err(TrySendError::Full(_)) => Err("ORACLE05_QUEUE_OVERFLOW".into()),
            Err(TrySendError::Disconnected(_)) => Err("ORACLE05_OBSERVER_DISCONNECTED".into()),
        }
    }
    pub fn finish(&mut self) -> Result<u64, String> {
        self.sender.take();
        if let Some(worker) = self.worker.take() {
            worker
                .join()
                .map_err(|_| "ORACLE05_OBSERVER_WRITER_PANIC")?;
        }
        if self.counters.failed.load(Ordering::Relaxed) > 0 {
            return Err("ORACLE05_OBSERVER_WRITE_FAILED".into());
        }
        Ok(self.counters.written.load(Ordering::Relaxed))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn event_packet_bounded() {
        assert!(encode(&serde_json::json!({"type":"wayland_button"}))
            .unwrap()
            .ends_with(b"\n"));
        assert!(encode(&serde_json::json!({"x":"b".repeat(2100)})).is_err());
    }
    #[test]
    fn refuses_insecure_parent_and_regular_files() {
        let folder = std::env::temp_dir().join(format!("oracle05-{}", std::process::id()));
        let _ = fs::remove_dir_all(&folder);
        fs::create_dir(&folder).unwrap();
        fs::set_permissions(&folder, fs::Permissions::from_mode(0o755)).unwrap();
        let file = folder.join("endpoint");
        fs::write(&file, b"x").unwrap();
        assert_eq!(
            SemanticPublisher::connect(&file, serde_json::json!({}))
                .err()
                .unwrap(),
            "ORACLE05_SOCKET_PARENT_MUST_BE_PRIVATE"
        );
        fs::set_permissions(&folder, fs::Permissions::from_mode(0o700)).unwrap();
        assert_eq!(
            SemanticPublisher::connect(&file, serde_json::json!({}))
                .err()
                .unwrap(),
            "ORACLE05_SOCKET_NOT_PRIVATE_UNIX_SOCKET"
        );
        fs::remove_dir_all(&folder).unwrap();
    }
    /// Optional cross-language IPC probe. The packet is SYNTHETIC and
    /// must never be advertised as an actual physical Wayland button.
    #[test]
    fn synthetic_rust_to_in_gnu6_node_runtime_if_requested() {
        let Ok(path) = std::env::var("GNU6_ORACLE05_SOCKET_TEST") else {
            return;
        };
        let mut client = SemanticPublisher::connect(
            Path::new(&path),
            serde_json::json!({
                "type":"hello","schema":"blobin.gpu05.semantic-witness.v1",
                "surface_token":"aaaaaaaaaaaa",
                "scene_frame_sequence":13,
                "scene_source":"in.gnu6-verified-scene-stream-v1",
                "coordinate_space":"surface_logical_pixels",
                "consumer_authority":"read_only_observation",
                "calibration_status":"fixture_declared_not_physically_verified"
            }),
        )
        .expect("test-only handshake");
        client
            .emit(&serde_json::json!({
                "type":"wayland_button","button":"BTN_LEFT",
                "timing_class":"unattested_wayland_software_input",
                "scene_frame_sequence":13,
                "wl_surface_token":"aaaaaaaaaaaa",
                "provider_id":"wayland-gpu05-owned-surface",
                "pointer_id":"primary",
                "provider_sequence":1,"pressed":true,
                "surface_x":1280.0,"surface_y":720.0,
                "coordinates_space":"surface_logical_pixels",
                "synthetic_test_packet":true
            }))
            .expect("test-only packet");
        assert_eq!(client.finish(), Ok(1));
    }
    #[test]
    fn sends_real_bytes_to_local_peer() {
        use std::os::unix::net::UnixListener;
        let folder = std::env::temp_dir().join(format!("oracle05-peer-{}", std::process::id()));
        let _ = fs::remove_dir_all(&folder);
        fs::create_dir(&folder).unwrap();
        fs::set_permissions(&folder, fs::Permissions::from_mode(0o700)).unwrap();
        let path = folder.join("observer.sock");
        let listener = UnixListener::bind(&path).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        let consumer = thread::spawn(move || {
            use std::io::{BufRead, BufReader};
            let (client, _) = listener.accept().unwrap();
            let mut lines = BufReader::new(client);
            let mut one = String::new();
            let mut two = String::new();
            lines.read_line(&mut one).unwrap();
            lines.read_line(&mut two).unwrap();
            (one, two)
        });
        let mut publisher =
            SemanticPublisher::connect(&path, serde_json::json!({"type":"hello"})).unwrap();
        publisher
            .emit(&serde_json::json!({"type":"wayland_button","provider_sequence":1}))
            .unwrap();
        assert_eq!(publisher.finish(), Ok(1));
        let (a, b) = consumer.join().unwrap();
        assert!(a.contains("hello") && b.contains("wayland_button"));
        fs::remove_dir_all(folder).unwrap();
    }
}
