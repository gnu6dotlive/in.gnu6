// Exact MXC v1 SDK reject-on-unsupported network probe; temporary marker only.
use std::{env,process};
use mxc_sdk::v1::policy::{FilesystemPolicy,NetworkPolicy,NetworkAction,NetworkEgressPolicy,NetworkIngressPolicy};
use mxc_sdk::v1::{self,ContainerRequest,Containment,WaitResult};
fn main(){
  let a:Vec<String>=env::args().collect();
  assert_eq!(a.len(),3,"mode + disposable directory");
  let mode=&a[1];let dir=&a[2];
  assert!(dir.contains("gnu6-mxc-network-only-"));
  assert!(mode=="default-deny"||mode=="egress-allow"||mode=="ingress-allow");
  let policy=match mode.as_str() {
    "default-deny"=>None,
    "egress-allow"=>Some(NetworkPolicy{egress:Some(NetworkEgressPolicy{
      default:Some(NetworkAction::Allow),..Default::default()}),
      ingress:Some(NetworkIngressPolicy{default:Some(NetworkAction::Deny),
        host_loopback:Some(NetworkAction::Deny)}),..Default::default()}),
    "ingress-allow"=>Some(NetworkPolicy{egress:Some(NetworkEgressPolicy{
      default:Some(NetworkAction::Deny),..Default::default()}),
      ingress:Some(NetworkIngressPolicy{default:Some(NetworkAction::Allow),
        host_loopback:Some(NetworkAction::Deny)}),..Default::default()}),
    _=>unreachable!()
  };
  let file=FilesystemPolicy{readwrite_paths:vec![dir.to_string()],..Default::default()};
  let req=ContainerRequest{containment:Containment::Process,filesystem:Some(file),network:policy,
    working_directory:Some(dir.clone()),timeout_ms:Some(6000),
    ..ContainerRequest::new("sh -c \"printf EXECUTED > marker.txt\"")};
  match v1::run(req,Default::default()) {
    Ok(out)=>{
      print!("{}",String::from_utf8_lossy(&out.stdout));
      eprint!("{}",String::from_utf8_lossy(&out.stderr));
      match out.outcome {
        WaitResult::Exited(code)=>{println!("MXC_EXIT={code}");process::exit(code)},
        WaitResult::TimedOut=>{eprintln!("MXC_TIMED_OUT");process::exit(124)}
      }
    }
    Err(err)=>{eprintln!("MXC_SDK_ERROR:{err}");process::exit(3)}
  }
}
