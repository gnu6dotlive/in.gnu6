//! Local-only Linux AF_UNIX SOCK_SEQPACKET, memfd and SCM_RIGHTS, no network listeners.
use crate::frame_protocol::{MAX_CONTROL, MAX_FRAME};
use std::{
    ffi::CString,
    fs::{self, File},
    io::{Seek, SeekFrom, Write},
    mem::size_of,
    os::{
        fd::{AsRawFd, FromRawFd, OwnedFd, RawFd},
        unix::{
            ffi::OsStrExt,
            fs::{MetadataExt, PermissionsExt},
        },
    },
    path::Path,
};

fn os_error(label: &str) -> String {
    format!("{label}: {}", std::io::Error::last_os_error())
}

pub fn listen(path: &Path) -> Result<OwnedFd, String> {
    if !path.is_absolute() || path.exists() {
        return Err("FRAME_SOCKET_INVALID_PATH".into());
    }
    let parent = path.parent().ok_or("FRAME_SOCKET_PARENT_MISSING")?;
    if !parent.exists() {
        fs::create_dir(parent).map_err(|e| e.to_string())?;
        fs::set_permissions(parent, fs::Permissions::from_mode(0o700))
            .map_err(|e| e.to_string())?;
    }
    let meta = fs::symlink_metadata(parent).map_err(|e| e.to_string())?;
    if !meta.file_type().is_dir()
        || meta.permissions().mode() & 0o777 != 0o700
        || meta.uid() != unsafe { libc::geteuid() }
        || meta.gid() != unsafe { libc::getegid() }
    {
        return Err("FRAME_SOCKET_PARENT_REQUIRES_PRIVATE_0700_OWNED_DIR".into());
    }
    let bytes = path.as_os_str().as_bytes();
    let mut addr: libc::sockaddr_un = unsafe { std::mem::zeroed() };
    if bytes.len() > addr.sun_path.len() - 1 {
        return Err("FRAME_UNIX_SOCKET_PATH_TOO_LONG".into());
    }
    addr.sun_family = libc::AF_UNIX as libc::sa_family_t;
    for (dst, src) in addr.sun_path.iter_mut().zip(bytes.iter()) {
        *dst = *src as libc::c_char;
    }
    let fd = unsafe { libc::socket(libc::AF_UNIX, libc::SOCK_SEQPACKET | libc::SOCK_CLOEXEC, 0) };
    if fd < 0 {
        return Err(os_error("FRAME_SOCKET_CREATE"));
    }
    let sock = unsafe { OwnedFd::from_raw_fd(fd) };
    let len =
        (std::mem::offset_of!(libc::sockaddr_un, sun_path) + bytes.len() + 1) as libc::socklen_t;
    if unsafe { libc::bind(sock.as_raw_fd(), (&raw const addr).cast(), len) } != 0 {
        return Err(os_error("FRAME_SOCKET_BIND"));
    }
    fs::set_permissions(path, fs::Permissions::from_mode(0o600)).map_err(|e| e.to_string())?;
    if unsafe { libc::listen(sock.as_raw_fd(), 8) } != 0 {
        return Err(os_error("FRAME_SOCKET_LISTEN"));
    }
    Ok(sock)
}

pub fn accept(sock: &OwnedFd) -> Result<OwnedFd, String> {
    let fd = unsafe {
        libc::accept4(
            sock.as_raw_fd(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            libc::SOCK_CLOEXEC,
        )
    };
    if fd < 0 {
        return Err(os_error("FRAME_SOCKET_ACCEPT"));
    }
    let accepted = unsafe { OwnedFd::from_raw_fd(fd) };
    let mut peer: libc::ucred = unsafe { std::mem::zeroed() };
    let mut length = size_of::<libc::ucred>() as libc::socklen_t;
    let result = unsafe {
        libc::getsockopt(
            fd,
            libc::SOL_SOCKET,
            libc::SO_PEERCRED,
            (&raw mut peer).cast(),
            &raw mut length,
        )
    };
    if result < 0 || length as usize != size_of::<libc::ucred>() {
        return Err("FRAME_PEER_CREDENTIALS_UNAVAILABLE".into());
    }
    if peer.uid != unsafe { libc::geteuid() } || peer.pid <= 0 {
        return Err("FRAME_PEER_UID_MISMATCH".into());
    }
    // A same-UID client must not monopolize the single-owner preview daemon
    // by connecting and never sending another control packet.
    let deadline = libc::timeval {
        tv_sec: 30,
        tv_usec: 0,
    };
    for option in [libc::SO_RCVTIMEO, libc::SO_SNDTIMEO] {
        if unsafe {
            libc::setsockopt(
                fd,
                libc::SOL_SOCKET,
                option,
                (&raw const deadline).cast(),
                size_of::<libc::timeval>() as libc::socklen_t,
            )
        } < 0
        {
            return Err(os_error("FRAME_SOCKET_TIMEOUT_SETUP"));
        }
    }
    Ok(accepted)
}

pub fn recv_packet(fd: RawFd) -> Result<Option<Vec<u8>>, String> {
    let mut bytes = vec![0u8; MAX_CONTROL + 1];
    let n = unsafe { libc::recv(fd, bytes.as_mut_ptr().cast(), bytes.len(), libc::MSG_TRUNC) };
    if n == 0 {
        return Ok(None);
    }
    if n < 0 {
        return Err(os_error("FRAME_RECV"));
    }
    if n as usize > MAX_CONTROL || n == 0 {
        return Err("FRAME_PACKET_TOO_LARGE".into());
    }
    bytes.truncate(n as usize);
    Ok(Some(bytes))
}

pub fn send_packet(fd: RawFd, body: &[u8], payload_fd: Option<RawFd>) -> Result<(), String> {
    if body.is_empty() || body.len() > MAX_CONTROL {
        return Err("FRAME_SEND_CONTROL_BOUNDS".into());
    }
    let mut iov = libc::iovec {
        iov_base: body.as_ptr() as *mut _,
        iov_len: body.len(),
    };
    let mut msg: libc::msghdr = unsafe { std::mem::zeroed() };
    msg.msg_iov = &raw mut iov;
    msg.msg_iovlen = 1;
    let mut ctrl = [0usize; 8]; // 64 bytes, sufficient for one aligned SCM_RIGHTS cmsghdr
    if let Some(send_fd) = payload_fd {
        let needed = unsafe { libc::CMSG_SPACE(size_of::<RawFd>() as u32) } as usize;
        if needed > size_of::<[usize; 8]>() {
            return Err("FRAME_SCM_BUFFER_INVALID".into());
        }
        msg.msg_control = ctrl.as_mut_ptr().cast();
        msg.msg_controllen = needed;
        let hdr = unsafe { libc::CMSG_FIRSTHDR(&msg) };
        if hdr.is_null() {
            return Err("FRAME_SCM_HEADER_INVALID".into());
        }
        unsafe {
            (*hdr).cmsg_level = libc::SOL_SOCKET;
            (*hdr).cmsg_type = libc::SCM_RIGHTS;
            (*hdr).cmsg_len = libc::CMSG_LEN(size_of::<RawFd>() as u32) as _;
            std::ptr::write(libc::CMSG_DATA(hdr).cast::<RawFd>(), send_fd);
        }
    }
    let n = unsafe { libc::sendmsg(fd, &msg, libc::MSG_NOSIGNAL) };
    if n < 0 {
        return Err(os_error("FRAME_SENDMSG"));
    }
    if n as usize != body.len() {
        return Err("FRAME_SHORT_SEQPACKET_SEND".into());
    }
    Ok(())
}

pub fn make_sealed_frame(pixels: &[u8]) -> Result<File, String> {
    if pixels.is_empty() || pixels.len() > MAX_FRAME {
        return Err("FRAME_BYTES_INVALID".into());
    }
    let name = CString::new("blobin-frame-rgba").expect("static memfd name");
    let fd =
        unsafe { libc::memfd_create(name.as_ptr(), libc::MFD_CLOEXEC | libc::MFD_ALLOW_SEALING) };
    if fd < 0 {
        return Err(os_error("FRAME_MEMFD_CREATE"));
    }
    let mut file = unsafe { File::from_raw_fd(fd) };
    file.write_all(pixels)
        .map_err(|e| format!("FRAME_MEMFD_WRITE {e}"))?;
    file.flush().map_err(|e| e.to_string())?;
    file.seek(SeekFrom::Start(0))
        .map_err(|_| "FRAME_MEMFD_REWIND_FAILED")?;
    let seals = libc::F_SEAL_WRITE | libc::F_SEAL_GROW | libc::F_SEAL_SHRINK | libc::F_SEAL_SEAL;
    if unsafe { libc::fcntl(fd, libc::F_ADD_SEALS, seals) } < 0 {
        return Err(os_error("FRAME_MEMFD_SEAL"));
    }
    let actual = unsafe { libc::fcntl(fd, libc::F_GET_SEALS) };
    if actual & seals != seals {
        return Err("FRAME_MEMFD_NOT_SEALED".into());
    }
    Ok(file)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sealed_memfd_readable_not_writable() {
        use std::{
            io::{Read, Seek, SeekFrom},
            os::fd::AsRawFd,
        };
        let f = make_sealed_frame(&[1, 2, 3, 255]).unwrap();
        let mut copy = f.try_clone().unwrap();
        copy.seek(SeekFrom::Start(0)).unwrap();
        let mut bytes = Vec::new();
        copy.read_to_end(&mut bytes).unwrap();
        assert_eq!(bytes, [1, 2, 3, 255]);
        assert_eq!(
            unsafe { libc::write(f.as_raw_fd(), [9u8].as_ptr().cast(), 1) },
            -1
        );
    }
    #[test]
    fn socket_requires_private_mode() {
        use std::time::{SystemTime, UNIX_EPOCH};
        let label = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("bfv1-{}-{label}", std::process::id()));
        std::fs::create_dir(&root).unwrap();
        let socket = root.join("s");
        std::fs::set_permissions(&root, fs::Permissions::from_mode(0o755)).unwrap();
        assert!(listen(&socket).is_err());
        std::fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
        let server = listen(&socket).unwrap();
        assert!(listen(&socket).is_err());
        drop(server);
        std::fs::remove_file(socket).unwrap();
        std::fs::remove_dir(root).unwrap();
    }
}
