//! Linux-specific content fingerprint + pinned file descriptor for native CUDA loading.
//! SHA-256 proves byte equality, not safety, signer identity, or sandbox isolation.
use sha2::{Digest, Sha256};
use std::fs::{File, OpenOptions};
use std::io::Read;
use std::os::fd::AsRawFd;
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProviderFingerprint {
    pub sha256: [u8; 32],
    pub len: u64,
    pub device_id: u64,
    pub inode: u64,
}
pub struct PinnedProvider {
    file: File,
    pub fingerprint: ProviderFingerprint,
}
impl PinnedProvider {
    /// The file descriptor remains open during dlopen to avoid pathname
    /// replacement between verification and opening the library.
    pub fn loader_path(&self) -> PathBuf {
        PathBuf::from(format!("/proc/self/fd/{}", self.file.as_raw_fd()))
    }
}
pub fn inspect(path: &Path) -> Result<PinnedProvider, String> {
    if !path.is_absolute() {
        return Err("CUDA04D PROVIDER PATH MUST BE ABSOLUTE".into());
    }
    // O_NOFOLLOW (Linux) disallows leaf symlinks. Ancestor paths are not
    // protected; a strict SHA-256 allowlist remains required for trust policy.
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(0o400000 | 0o4000)
        .open(path)
        .map_err(|e| format!("CUDA04D OPEN PROVIDER: {e}"))?;
    let before = file
        .metadata()
        .map_err(|e| format!("CUDA04D METADATA: {e}"))?;
    if !before.is_file() || before.len() == 0 || before.len() > 64 * 1024 * 1024 {
        return Err("CUDA04D INVALID PROVIDER SIZE OR TYPE".into());
    }
    let mut hash = Sha256::new();
    let mut reader = &file;
    let mut bytes = 0u64;
    let mut buffer = [0u8; 65536];
    loop {
        let n = reader
            .read(&mut buffer)
            .map_err(|e| format!("CUDA04D READ PROVIDER: {e}"))?;
        if n == 0 {
            break;
        }
        bytes += n as u64;
        if bytes > before.len() {
            return Err("CUDA04D PROVIDER GROWTH".into());
        }
        hash.update(&buffer[..n]);
    }
    let after = file
        .metadata()
        .map_err(|e| format!("CUDA04D METADATA: {e}"))?;
    if bytes != before.len()
        || after.len() != before.len()
        || before.mtime() != after.mtime()
        || before.mtime_nsec() != after.mtime_nsec()
        || before.ino() != after.ino()
        || before.dev() != after.dev()
    {
        return Err("CUDA04D PROVIDER MUTATED DURING INSPECTION".into());
    }
    Ok(PinnedProvider {
        file,
        fingerprint: ProviderFingerprint {
            sha256: hash.finalize().into(),
            len: bytes,
            device_id: after.dev(),
            inode: after.ino(),
        },
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        os::unix::fs::symlink,
        time::{SystemTime, UNIX_EPOCH},
    };
    fn temp() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir =
            std::env::temp_dir().join(format!("blobin-cuda04d-{}-{nonce}", std::process::id()));
        fs::create_dir(&dir).unwrap();
        dir
    }
    #[test]
    fn fingerprint_detects_content_replacement_and_keeps_fd_pinned() {
        let dir = temp();
        let file = dir.join("provider.so");
        fs::write(&file, b"first-payload").unwrap();
        let pinned = inspect(&file).unwrap();
        assert!(pinned.loader_path().is_absolute());
        assert!(pinned.loader_path().exists());
        let previous = pinned.fingerprint;
        fs::write(&file, b"other-payload").unwrap();
        let current = inspect(&file).unwrap().fingerprint;
        assert_ne!(previous.sha256, current.sha256);
        fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn leaf_symlink_and_relative_path_are_rejected() {
        let dir = temp();
        let file = dir.join("real");
        fs::write(&file, b"x").unwrap();
        let link = dir.join("alias");
        symlink(&file, &link).unwrap();
        assert!(inspect(&link).is_err());
        assert!(inspect(Path::new("relative.so")).is_err());
        fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn missing_and_zero_size_are_denied() {
        let dir = temp();
        let file = dir.join("empty");
        fs::write(&file, b"").unwrap();
        assert!(inspect(&file).is_err());
        assert!(inspect(&dir.join("missing")).is_err());
        fs::remove_dir_all(dir).unwrap();
    }
}
