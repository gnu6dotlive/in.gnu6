// Disposable MXC SDK v1.0 Rust fixture, no host or production mutations.
use std::env;
use std::process;
use mxc_sdk::v1::policy::FilesystemPolicy;
use mxc_sdk::v1::{self, ContainerRequest, Containment, WaitResult};

fn main() {
    let args: Vec<String> = env::args().collect();
    assert_eq!(args.len(), 3, "usage: mxc-runtime-fixture ro|rw <disposable-dir>");
    let mode = args[1].as_str();
    assert!(mode == "ro" || mode == "rw");
    let dir = &args[2];
    assert!(dir.contains("gnu6-mxc-only-"), "refusing non-disposable target");
    let policy = FilesystemPolicy {
        readonly_paths: if mode == "ro" { vec![dir.clone()] } else { vec![] },
        readwrite_paths: if mode == "rw" { vec![dir.clone()] } else { vec![] },
        ..Default::default()
    };
    let shell = if mode == "ro" {
        "sh -c \"cat sentinel.txt; if printf altered > marker.txt 2>/dev/null; then echo WRITE_ALLOWED; else echo WRITE_DENIED; fi\""
    } else {
        "sh -c \"cat sentinel.txt; if printf allowed > marker.txt 2>/dev/null; then echo WRITE_ALLOWED; else echo WRITE_DENIED; fi\""
    };
    let request = ContainerRequest { containment: Containment::Process,
        filesystem: Some(policy), working_directory: Some(dir.clone()),
        timeout_ms: Some(7_000), ..ContainerRequest::new(shell) };
    match v1::run(request, Default::default()) {
        Ok(result) => {
            print!("{}", String::from_utf8_lossy(&result.stdout));
            eprint!("{}", String::from_utf8_lossy(&result.stderr));
            for warning in result.warnings { eprintln!("MXC_WARNING: {warning}"); }
            match result.outcome {
                WaitResult::Exited(code) => { println!("MXC_CHILD_EXIT={code}"); process::exit(code); }
                WaitResult::TimedOut => { eprintln!("MXC_TIMEOUT"); process::exit(124); }
            }
        }
        Err(err) => { eprintln!("MXC_SDK_ERROR: {err}"); process::exit(3); }
    }
}
