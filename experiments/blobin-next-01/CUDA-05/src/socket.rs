//! Linux peer-credential and private Unix-socket gate.
//! SO_PEERCRED authenticates a UID/PID at connect time, not an individual agent
//! or a distinction between programs sharing that same UID.
use std::os::fd::AsRawFd;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixStream;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Peer {
    pub pid: i32,
    pub uid: u32,
    pub gid: u32,
}
pub fn credentials(stream: &UnixStream) -> Result<Peer, String> {
    let mut c = libc::ucred {
        pid: 0,
        uid: 0,
        gid: 0,
    };
    let mut len = std::mem::size_of::<libc::ucred>() as libc::socklen_t;
    // SAFETY: getsockopt writes into a valid initialized ucred buffer; fd is live.
    let rc = unsafe {
        libc::getsockopt(
            stream.as_raw_fd(),
            libc::SOL_SOCKET,
            libc::SO_PEERCRED,
            (&raw mut c).cast(),
            &raw mut len,
        )
    };
    if rc != 0 || len as usize != std::mem::size_of::<libc::ucred>() {
        return Err("CUDA05B PEER CREDENTIALS UNAVAILABLE".into());
    }
    Ok(Peer {
        pid: c.pid,
        uid: c.uid,
        gid: c.gid,
    })
}
pub fn authorize(stream: &UnixStream, allowed_uid: u32) -> Result<Peer, String> {
    let peer = credentials(stream)?;
    if peer.uid != allowed_uid {
        return Err("CUDA05B PEER UID NOT AUTHORIZED".into());
    }
    Ok(peer)
}
pub fn validate_socket_destination(path: &Path) -> Result<(), String> {
    if !path.is_absolute() || path.exists() {
        return Err("CUDA05B SOCKET PATH MUST BE ABSOLUTE AND NONEXISTENT".into());
    }
    let parent = path.parent().ok_or("CUDA05B SOCKET PARENT MISSING")?;
    let meta = std::fs::symlink_metadata(parent).map_err(|e| e.to_string())?;
    if !meta.file_type().is_dir() || meta.permissions().mode() & 0o077 != 0 {
        return Err("CUDA05B SOCKET PARENT MUST BE PRIVATE DIRECTORY".into());
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::net::UnixStream;
    #[test]
    fn peer_cred_accepts_expected_uid_rejects_other() {
        let (a, _b) = UnixStream::pair().unwrap();
        let peer = credentials(&a).unwrap();
        assert!(peer.pid > 0);
        assert_eq!(authorize(&a, peer.uid).unwrap().uid, peer.uid);
        assert!(authorize(&a, peer.uid.wrapping_add(1)).is_err());
    }
    #[test]
    fn socket_path_preconditions_fail_closed() {
        assert!(validate_socket_destination(Path::new("relative")).is_err());
        assert!(validate_socket_destination(Path::new("/tmp/cuda05b.sock")).is_err());
    }
}
