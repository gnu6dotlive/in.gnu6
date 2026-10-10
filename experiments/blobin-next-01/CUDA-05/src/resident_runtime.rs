//! Opt-in GPU-resident child transport. No user-facing socket or authority lives here.
//! Supervisor must verify executable digest, authorize TTL and revoke on failure.
use crate::frame_protocol::{MAX_CONTROL, MAX_FRAME};
use std::{
    io::{ErrorKind, Read, Write},
    os::fd::AsRawFd,
    path::Path,
    process::{Child, ChildStdin, Command, Stdio},
    sync::mpsc::{self, Receiver},
    thread,
    time::{Duration, Instant},
};
pub struct ResidentWorker {
    child: Child,
    input: ChildStdin,
    responses: Receiver<Result<Vec<u8>, String>>,
    born: Instant,
    used: u64,
    budget: u64,
    ttl: Duration,
    pub generation: u64,
}
impl ResidentWorker {
    pub fn launch(
        binary: &Path,
        backend: &str,
        lib: Option<&Path>,
        hash: Option<[u8; 32]>,
        ttl: Duration,
        budget: u64,
        generation: u64,
    ) -> Result<Self, String> {
        if !binary.is_absolute()
            || budget == 0
            || budget > 240
            || ttl < Duration::from_secs(1)
            || ttl > Duration::from_secs(120)
        {
            return Err("RESIDENT_WORKER_POLICY_INVALID".into());
        }
        let mut cmd = Command::new(binary);
        let hash_text = hash
            .map(|h| h.iter().map(|v| format!("{v:02x}")).collect::<String>())
            .unwrap_or_else(|| "-".to_string());
        cmd.arg(backend)
            .arg(lib.map(|p| p.to_str().unwrap_or("")).unwrap_or("-"))
            .arg(hash_text)
            .arg(budget.to_string())
            .arg(ttl.as_millis().to_string());
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        // The worker inherits the invoking desktop UID by design; do not call this
        // a privilege boundary. The process is independently killable/revocable.
        let mut child = cmd.spawn().map_err(|_| "RESIDENT_WORKER_SPAWN_FAILED")?;
        let input = child
            .stdin
            .take()
            .ok_or("RESIDENT_WORKER_STDIN_UNAVAILABLE")?;
        // Nonblocking request writes prevent a wedged worker from monopolizing
        // the daemon before recv_timeout can ever run.
        let old_flags = unsafe { libc::fcntl(input.as_raw_fd(), libc::F_GETFL) };
        if old_flags < 0
            || unsafe {
                libc::fcntl(
                    input.as_raw_fd(),
                    libc::F_SETFL,
                    old_flags | libc::O_NONBLOCK,
                )
            } < 0
        {
            let _ = child.kill();
            let _ = child.wait();
            return Err("RESIDENT_STDIN_NONBLOCK_SETUP_FAILED".into());
        }
        // pidfd ties TTL expiry to the exact process identity (no PID-reuse race).
        let pidfd =
            unsafe { libc::syscall(libc::SYS_pidfd_open, child.id() as libc::pid_t, 0) } as i32;
        if pidfd < 0 {
            let _ = child.kill();
            let _ = child.wait();
            return Err("RESIDENT_PIDFD_REQUIRED".into());
        }
        thread::spawn(move || {
            thread::sleep(ttl);
            unsafe {
                libc::syscall(
                    libc::SYS_pidfd_send_signal,
                    pidfd,
                    libc::SIGKILL,
                    std::ptr::null::<libc::siginfo_t>(),
                    0,
                );
                libc::close(pidfd);
            }
        });
        let mut output = child
            .stdout
            .take()
            .ok_or("RESIDENT_WORKER_STDOUT_UNAVAILABLE")?;
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || loop {
            let mut prefix = [0u8; 4];
            if output.read_exact(&mut prefix).is_err() {
                break;
            }
            let n = u32::from_le_bytes(prefix) as usize;
            if n == 0 || n > MAX_FRAME {
                let _ = tx.send(Err("RESIDENT_RESPONSE_LENGTH_INVALID".into()));
                break;
            }
            let mut bytes = vec![0u8; n];
            if output.read_exact(&mut bytes).is_err() {
                let _ = tx.send(Err("RESIDENT_RESPONSE_TRUNCATED".into()));
                break;
            }
            if tx.send(Ok(bytes)).is_err() {
                break;
            }
        });
        Ok(Self {
            child,
            input,
            responses: rx,
            born: Instant::now(),
            used: 0,
            budget,
            ttl,
            generation,
        })
    }
    fn send_bounded(input: &mut ChildStdin, body: &[u8], deadline: Instant) -> Result<(), String> {
        let mut position = 0;
        while position < body.len() {
            if Instant::now() >= deadline {
                return Err("RESIDENT_WRITE_TIMEOUT".into());
            }
            match input.write(&body[position..]) {
                Ok(0) => return Err("RESIDENT_WRITE_ZERO".into()),
                Ok(n) => position += n,
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(1))
                }
                Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
                Err(_) => return Err("RESIDENT_WRITE_ERROR".into()),
            }
        }
        Ok(())
    }
    pub fn expired(&mut self) -> bool {
        self.used >= self.budget
            || self.born.elapsed() >= self.ttl
            || self.child.try_wait().map_or(true, |v| v.is_some())
    }
    pub fn render(
        &mut self,
        json: &[u8],
        expected: usize,
        limit: Duration,
    ) -> Result<Vec<u8>, String> {
        if self.expired() {
            return Err("RESIDENT_WORKER_LEASE_EXPIRED".into());
        }
        if json.is_empty() || json.len() > MAX_CONTROL || expected == 0 || expected > MAX_FRAME {
            return Err("RESIDENT_REQUEST_BOUNDS".into());
        }
        let deadline = Instant::now() + limit;
        Self::send_bounded(
            &mut self.input,
            &(json.len() as u32).to_le_bytes(),
            deadline,
        )?;
        Self::send_bounded(&mut self.input, json, deadline)?;
        let reply = self
            .responses
            .recv_timeout(limit)
            .map_err(|_| "RESIDENT_RESPONSE_TIMEOUT_OR_EOF")??;
        if reply.len() != expected {
            return Err("RESIDENT_FRAME_SIZE_WRONG".into());
        }
        self.used += 1;
        Ok(reply)
    }
}
impl Drop for ResidentWorker {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
