//! GPU-04 local same-UID, explicitly opt-in, experimental scene delta ingress.
//! A dedicated bounded reader thread keeps networking off the render loop.
use std::{
    fs,
    io::{BufRead, BufReader, Read},
    os::{
        fd::AsRawFd,
        unix::{
            fs::PermissionsExt,
            net::{UnixListener, UnixStream},
        },
    },
    path::{Path, PathBuf},
    sync::mpsc::{self, Receiver, SyncSender, TrySendError},
    thread,
    time::Duration,
};
const MAX_MESSAGE: usize = 8192;

pub struct SocketGuard {
    path: PathBuf,
}
impl Drop for SocketGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}
fn peer_is_current_uid(stream: &UnixStream) -> bool {
    let fd = stream.as_raw_fd();
    let mut peer = libc::ucred {
        pid: 0,
        uid: 0,
        gid: 0,
    };
    let mut size = std::mem::size_of::<libc::ucred>() as libc::socklen_t;
    // SAFETY: fd is owned by stream; peer and size are properly sized, mutable stack storage.
    let result = unsafe {
        libc::getsockopt(
            fd,
            libc::SOL_SOCKET,
            libc::SO_PEERCRED,
            (&raw mut peer).cast(),
            &raw mut size,
        )
    };
    result == 0
        && size as usize == std::mem::size_of::<libc::ucred>()
        && peer.uid == unsafe { libc::geteuid() }
}
fn handle(mut stream: UnixStream, tx: &SyncSender<Vec<u8>>) {
    if !peer_is_current_uid(&stream) {
        eprintln!("GPU04_PEER_UID_REJECTED");
        return;
    }
    let _ = stream.set_read_timeout(Some(Duration::from_secs(1)));
    let mut reader = BufReader::new(&mut stream);
    loop {
        let mut line = Vec::new();
        let next = reader
            .by_ref()
            .take((MAX_MESSAGE + 2) as u64)
            .read_until(b'\n', &mut line);
        match next {
            Ok(0) => break,
            Ok(_) if line.last() != Some(&b'\n') || line.len() > MAX_MESSAGE + 1 => {
                eprintln!("GPU04_OVERSIZE_OR_UNTERMINATED_PACKET_REJECTED");
                break;
            }
            Ok(_) => {
                line.pop();
                match tx.try_send(line) {
                    Ok(()) => {}
                    Err(TrySendError::Full(_)) => eprintln!("GPU04_QUEUE_FULL_PACKET_DROPPED"),
                    Err(TrySendError::Disconnected(_)) => break,
                }
            }
            Err(e)
                if e.kind() == std::io::ErrorKind::WouldBlock
                    || e.kind() == std::io::ErrorKind::TimedOut =>
            {
                break
            }
            Err(e) => {
                eprintln!("GPU04_IPC_READ_ERROR {e}");
                break;
            }
        }
    }
}
pub fn listen(path: &Path) -> Result<(Receiver<Vec<u8>>, SocketGuard), String> {
    if path.exists() {
        return Err("GPU04_SOCKET_ALREADY_EXISTS".into());
    }
    let parent = path.parent().ok_or("GPU04_SOCKET_REQUIRES_ABSOLUTE_PATH")?;
    if !path.is_absolute() {
        return Err("GPU04_SOCKET_REQUIRES_ABSOLUTE_PATH".into());
    }
    let info = fs::metadata(parent).map_err(|e| format!("GPU04_SOCKET_PARENT_UNAVAILABLE {e}"))?;
    if !info.is_dir() || info.permissions().mode() & 0o077 != 0 {
        return Err("GPU04_PARENT_MUST_BE_PRIVATE_0700".into());
    }
    let listener = UnixListener::bind(path).map_err(|e| format!("GPU04_SOCKET_BIND_FAILED {e}"))?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))
        .map_err(|e| format!("GPU04_SOCKET_MODE_FAILED {e}"))?;
    listener
        .set_nonblocking(true)
        .map_err(|e| format!("GPU04_SOCKET_NONBLOCKING_FAILED {e}"))?;
    let (tx, rx) = mpsc::sync_channel(16);
    thread::Builder::new()
        .name("blobin-gpu04-local-deltas".into())
        .spawn(move || loop {
            match listener.accept() {
                Ok((stream, _)) => handle(stream, &tx),
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(12));
                }
                Err(e) => {
                    eprintln!("GPU04_SOCKET_ACCEPT_ERROR {e}");
                    break;
                }
            }
        })
        .map_err(|e| format!("GPU04_SOCKET_THREAD_FAILED {e}"))?;
    Ok((
        rx,
        SocketGuard {
            path: path.to_owned(),
        },
    ))
}
