// W15 lab-only adapter: sealed authority-core ExecutionGrant gates Deno IPC.
// NOT a production broker: test-support enabled for disposable fixtures only.
use gnosix_authority_core::{
    AuthorityEpochSource, ManualTestClock, ResourceScope, ScopedCapability,
    SystemAuthorityFactory, TrustedMonotonicClock,
};
use serde::{Deserialize, Serialize};
use std::{env, fs, io::{BufRead, BufReader, Write}, os::unix::net::{UnixListener, UnixStream},
          os::unix::fs::PermissionsExt, os::fd::AsRawFd, process::Command};
#[derive(Deserialize, Debug)]
struct Request {
    v:u32, pid:u32, id:u64, permission:String, value:Option<String>
}
#[derive(Serialize)]
struct Response {id:u64, result:&'static str}
#[derive(Serialize)]
struct Receipt {
    case_name:String, decision:String, request_seen:bool,
    peer_pid_matches:bool, peer_uid_matches:bool, request_pid_matches:bool,
    stdout:String, stderr:String, exit_code:i32,
    marker_exists:bool, marker_contents:Option<String>,
    grant_issued:bool, scope_valid:bool
}
fn peer_identity(stream:&UnixStream)->Option<(u32,u32)>{
    let mut cred=libc::ucred{pid:0,uid:0,gid:0};
    let mut len=std::mem::size_of::<libc::ucred>() as libc::socklen_t;
    let rc=unsafe{libc::getsockopt(stream.as_raw_fd(),
        libc::SOL_SOCKET,libc::SO_PEERCRED,&mut cred as *mut _ as *mut _,&mut len)};
    if rc==0 && len as usize==std::mem::size_of::<libc::ucred>() && cred.pid>0 {Some((cred.pid as u32,cred.uid))} else {None}
}
fn run() -> Result<Receipt,Box<dyn std::error::Error>> {
    let a:Vec<String>=env::args().collect();
    assert_eq!(a.len(),5,"usage: <case> <disposable-dir> <deno-exe> <script>");
    let case=&a[1];let base=&a[2];let deno=&a[3];let script=&a[4];
    assert!(base.contains("gnu6-w15-only-"));
    assert!(deno.ends_with("/deno-v2.9.7/runtime/deno"));
    assert!(script.ends_with("composite_deno_fixture.ts"));
    let dir=fs::canonicalize(base)?;
    assert!(dir.is_dir());
    let good=dir.join("marker.txt");
    let alias=dir.join("alias.txt");
    if case=="symlink_alias" {std::os::unix::fs::symlink(&good,&alias)?;}
    let target=if case=="symlink_alias" {&alias} else {&good};
    let target_value=target.to_str().ok_or("invalid target path")?.to_string();
    let authorized_value=good.to_str().ok_or("invalid authorized path")?.to_string();
    let cap=ScopedCapability{kind:"fs".into(),
        scope:ResourceScope::Filesystem{canonical_path:authorized_value.clone()},
        operations:vec![if case=="wrong_operation" {"read".into()} else {"write".into()}],
        max_ttl_seconds:30};
    let parent=ScopedCapability{kind:"fs".into(),
        scope:ResourceScope::Filesystem{canonical_path:dir.to_string_lossy().into_owned()},
        operations:vec!["read".into(),"write".into()],max_ttl_seconds:60};
    let epoch=AuthorityEpochSource::new(1);
    let manual=ManualTestClock::new(100);
    let clock=TrustedMonotonicClock::test_clock(manual.clone());
    let (authority,revoker)=SystemAuthorityFactory::create_engine_for_testing(vec![parent],&epoch,clock);
    let digest=format!("deno.write:{}",authorized_value);
    let identity=format!("fs:{}",authorized_value);
    let wanted=if case=="scope_escape" {ScopedCapability{
        scope:ResourceScope::Filesystem{canonical_path:"/not-delegable/outside".into()},
        ..cap.clone()
    }} else {cap.clone()};
    let ttl=if case=="ttl_exceeds" {99} else {3};
    let grant=authority.authorize_and_issue("W15-grant-one".into(),
        wanted,digest.clone(),identity.clone(),ttl);
    let grant_issued=grant.is_ok();
    if case=="expired" {manual.advance_seconds(4);}
    if case=="revoked" {revoker.revoke_all();}
    if case=="replay" {grant.as_ref().expect("replay grant").consume_atomic(&digest,&identity)?;}
    let sock=dir.join("broker.sock");
    let listener=UnixListener::bind(&sock)?;
    fs::set_permissions(&sock,fs::Permissions::from_mode(0o600))?;
    let child=Command::new(deno)
        .args(["run","--no-config","--no-prompt",script,&target_value])
        .env("DENO_PERMISSION_BROKER_PATH",&sock)
        .env("DENO_DIR",dir.join("deno_cache"))
        .env("DENO_NO_UPDATE_CHECK","1")
        .stdout(std::process::Stdio::piped()).stderr(std::process::Stdio::piped())
        .spawn()?;
    let expected_pid=child.id();
    let (mut peer,_)=listener.accept()?;
    peer.set_read_timeout(Some(std::time::Duration::from_secs(6)))?;
    let kernel_identity=peer_identity(&peer);
    let peer_matches=kernel_identity.map(|x|x.0)==Some(if case=="wrong_peer" {expected_pid+1} else {expected_pid});
    let peer_uid_matches=kernel_identity.map(|x|x.1)==Some(unsafe{libc::geteuid()});
    let mut request_raw=String::new();
    BufReader::new(peer.try_clone()?).read_line(&mut request_raw)?;
    let request:Request=serde_json::from_str(&request_raw)?;
    let request_pid_matches=request.pid==expected_pid;
    let request_seen=true;
    let requested_path=request.value.as_ref().map(String::as_str).unwrap_or("");
    let lexical_expected=requested_path==authorized_value;
    let req_path=std::path::Path::new(requested_path);
    let parent_canonical=req_path.parent().and_then(|p|fs::canonicalize(p).ok());
    let final_symlink=fs::symlink_metadata(req_path).map(|m|m.file_type().is_symlink()).unwrap_or(false);
    let scope_valid=lexical_expected && parent_canonical.as_deref()==Some(dir.as_path()) && !final_symlink;
    let valid_protocol=request.v==1 && request.permission=="write" && request.id>0;
    let operation_granted=grant.as_ref().ok().map(|g|
        g.capability().kind=="fs" &&
        g.capability().operations.contains(&request.permission) &&
        g.capability().scope==ResourceScope::Filesystem{canonical_path:authorized_value.clone()}
    ).unwrap_or(false);
    let action_arg=if case=="wrong_digest" {"deno.write:tampered"} else {&digest};
    let resource_arg=if case=="wrong_identity" {"fs:tampered"} else {&identity};
    let is_valid=valid_protocol && peer_matches && peer_uid_matches && request_pid_matches && scope_valid && operation_granted
        && grant.as_ref().map(|g|g.consume_atomic(action_arg,resource_arg).is_ok()).unwrap_or(false);
    let decision=if is_valid {"allow"} else {"deny"};
    let response=Response{id:request.id,result:decision};
    peer.write_all(format!("{}\n",serde_json::to_string(&response)?).as_bytes())?;
    drop(peer);
    let output=child.wait_with_output()?;
    let stdout=String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr=String::from_utf8_lossy(&output.stderr).into_owned();
    let marker_exists=target.exists();
    let marker_contents=if marker_exists {fs::read_to_string(target).ok()} else {None};
    let receipt=Receipt {case_name:case.clone(),decision:decision.into(),
        request_seen,peer_pid_matches:peer_matches,peer_uid_matches,request_pid_matches,
        stdout:stdout.chars().take(600).collect(),
        stderr:stderr.chars().take(600).collect(),
        exit_code:output.status.code().unwrap_or(-1),
        marker_exists,marker_contents,grant_issued,scope_valid};
    Ok(receipt)
}
fn main(){
    match run(){
        Ok(receipt)=>println!("{}",serde_json::to_string(&receipt).expect("serialize")),
        Err(error)=>{eprintln!("W15_LAB_ERROR:{error}");std::process::exit(2);}
    }
}
