// MXC SDK v1 Process backend × Deno localhost network policy fixture.
use std::{env,process};
use mxc_sdk::v1::policy::{FilesystemPolicy,NetworkAction,NetworkEgressPolicy,NetworkIngressPolicy,NetworkPolicy};
use mxc_sdk::v1::{self,ContainerRequest,Containment,WaitResult};
fn main() {
  let a:Vec<String>=env::args().collect();
  assert_eq!(a.len(),7,"<mxc-allow|mxc-deny> <deno-allow|deno-deny> <tmp-dir> <port> <deno-bin> <script>");
  let (mode,perm,dir,port,deno,script)=(&a[1],&a[2],&a[3],&a[4],&a[5],&a[6]);
  assert!(mode=="mxc-allow"||mode=="mxc-deny");
  assert!(perm=="deno-allow"||perm=="deno-deny");
  assert!(dir.contains("gnu6-mxc-deno-only-"));
  assert!(port.parse::<u16>().is_ok());
  assert!(deno.contains("/upstream-rd-01/deno-v2.9.7/runtime/deno"));
  assert!(script.ends_with("composite_deno_net.ts"));
  let fs=FilesystemPolicy{readonly_paths:vec![dir.clone(),deno.clone(),script.clone()],..Default::default()};
  let network=if mode=="mxc-deny" {
    Some(NetworkPolicy{egress:Some(NetworkEgressPolicy{default:Some(NetworkAction::Deny),..Default::default()}),
      ingress:Some(NetworkIngressPolicy{default:Some(NetworkAction::Deny),host_loopback:Some(NetworkAction::Deny)}),
      ..Default::default()})
  } else {None};
  let flag=if perm=="deno-allow" {format!("--allow-net=127.0.0.1:{port}")} else {"--no-prompt".to_string()};
  let url=format!("http://127.0.0.1:{port}/");
  let command=format!("DENO_NO_UPDATE_CHECK=1 DENO_DIR=/tmp/gnu6-deno-net {deno} run --no-config --no-prompt {flag} {script} {url}");
  let req=ContainerRequest{containment:Containment::Process,filesystem:Some(fs),network,
    working_directory:Some(dir.clone()),timeout_ms:Some(10_000),..ContainerRequest::new(command)};
  match v1::run(req,Default::default()) {
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
