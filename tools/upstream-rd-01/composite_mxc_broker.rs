// MXC SDK v1.0.0 hosts a Deno 2.9.7 permission fixture.
// Requires a disposable directory. No other repository or production mutation.
use std::{env,process};
use mxc_sdk::v1::policy::FilesystemPolicy;
use mxc_sdk::v1::{self,ContainerRequest,Containment,WaitResult};
fn main() {
  let args:Vec<String>=env::args().collect();
  assert_eq!(args.len(),6,"usage: fixture <ro|rw> <dir> <deno-bin> <script> <broker-sock>");
  let(mode,dir,deno,script,sock)=(&args[1],&args[2],&args[3],&args[4],&args[5]);
  assert!(mode=="ro"||mode=="rw");
  assert!(sock.ends_with("broker.sock")&&sock.starts_with(dir));
  assert!(dir.contains("gnu6-mxc-deno-only-"));
  assert!(deno.contains("/upstream-rd-01/deno-v2.9.7/runtime/deno"));
  assert!(script.ends_with("composite_deno_fixture.ts"));
  let mut ro=vec![deno.clone(),script.clone()];
  if mode=="ro" {ro.push(dir.clone());}
  let rw=if mode=="rw" {vec![dir.clone()]} else {vec![]};
  let filesystem=FilesystemPolicy{readonly_paths:ro,readwrite_paths:rw,..Default::default()};
  let command=format!("DENO_NO_UPDATE_CHECK=1 DENO_DIR=/tmp/gnu6-deno-only DENO_PERMISSION_BROKER_PATH={sock} {deno} run --no-config --no-prompt {script} {dir}/marker.txt");
  let request=ContainerRequest{containment:Containment::Process,filesystem:Some(filesystem),
    working_directory:Some(dir.clone()),timeout_ms:Some(10_000),
    ..ContainerRequest::new(command)};
  match v1::run(request,Default::default()) {
    Ok(result)=>{
      print!("{}",String::from_utf8_lossy(&result.stdout));
      eprint!("{}",String::from_utf8_lossy(&result.stderr));
      for w in result.warnings {eprintln!("MXC_WARNING:{w}");}
      match result.outcome {
        WaitResult::Exited(code)=>{println!("MXC_CHILD_EXIT={code}");process::exit(code);},
        WaitResult::TimedOut=>{eprintln!("MXC_TIMEOUT");process::exit(124);}
      }
    }
    Err(err)=>{eprintln!("MXC_SDK_ERROR:{err}");process::exit(3);}
  }
}
