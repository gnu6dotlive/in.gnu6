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
/// Strict lab-only group-accessible socket destination for distinct host UIDs.
/// Owner and group must be those of the unprivileged service, not the agent.
pub fn validate_group_socket_destination(path: &Path) -> Result<(), String> {
    if !path.is_absolute() || path.exists() {
        return Err("CUDA05F SOCKET PATH MUST BE ABSOLUTE AND NONEXISTENT".into());
    }
    let parent = path.parent().ok_or("CUDA05F SOCKET PARENT MISSING")?;
    let meta = std::fs::symlink_metadata(parent).map_err(|e| e.to_string())?;
    use std::os::unix::fs::MetadataExt;
    if !meta.file_type().is_dir()
        || meta.permissions().mode() & 0o777 != 0o710
        || meta.uid() != unsafe { libc::geteuid() }
        || meta.gid() != unsafe { libc::getegid() }
    {
        return Err("CUDA05F SOCKET DIRECTORY NOT OWNED BY SERVICE AT MODE 0710".into());
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
    fn distinct_principal_socket_requires_exact_owner_group_and_mode() {
        use std::{
            fs,
            time::{SystemTime, UNIX_EPOCH},
        };
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("blobin-05f-{}-{nonce}", std::process::id()));
        fs::create_dir(&root).unwrap();
        fs::set_permissions(&root, fs::Permissions::from_mode(0o710)).unwrap();
        let destination = root.join("service.sock");
        assert!(validate_group_socket_destination(&destination).is_ok());
        fs::set_permissions(&root, fs::Permissions::from_mode(0o770)).unwrap();
        assert!(validate_group_socket_destination(&destination).is_err());
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
        assert!(validate_group_socket_destination(&destination).is_err());
        fs::set_permissions(&root, fs::Permissions::from_mode(0o710)).unwrap();
        let link = root.with_extension("alias");
        std::os::unix::fs::symlink(&root, &link).unwrap();
        assert!(validate_group_socket_destination(&link.join("service.sock")).is_err());
        fs::remove_file(&link).unwrap();
        fs::remove_dir(&root).unwrap();
    }
    #[test]
    fn socket_path_preconditions_fail_closed() {
        assert!(validate_socket_destination(Path::new("relative")).is_err());
        assert!(validate_socket_destination(Path::new("/tmp/cuda05b.sock")).is_err());
    }
}
