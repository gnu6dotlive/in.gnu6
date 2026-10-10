#!/usr/bin/env python3
"""Opt-in BLOB-FRAME-v1 resident daemon test, isolated from systemd/live socket."""
import argparse,hashlib,json,pathlib,socket,subprocess,tempfile,time,sys,statistics
sys.path.insert(0,str(pathlib.Path(__file__).resolve().parent))
from qualify_frame_bridge import talk,frame
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 ap=argparse.ArgumentParser()
 ap.add_argument('--bin-dir',type=pathlib.Path,required=True)
 ap.add_argument('--cuda-lib',type=pathlib.Path,required=True)
 ap.add_argument('--receipt',type=pathlib.Path)
 args=ap.parse_args()
 b=args.bin_dir;lib=args.cuda_lib
 names=["blobin_renderd","blobin_frame_worker","blobin_resident_worker","worker05c"]
 for n in names:assert (b/n).is_file(),n
 with tempfile.TemporaryDirectory(prefix='blobin-resident-e2e-') as folder:
  sock=pathlib.Path(folder)/"bridge.sock"
  cmd=[str(b/'blobin_renderd'),'--socket',str(sock),
    '--frame-worker',str(b/'blobin_frame_worker'),
    '--frame-sha256',sha(b/'blobin_frame_worker'),
    '--gate-worker',str(b/'worker05c'),'--gate-sha256',sha(b/'worker05c'),
    '--cuda-lib',str(lib),'--cuda-sha256',sha(lib),
    '--resident-worker',str(b/'blobin_resident_worker'),
    '--resident-sha256',sha(b/'blobin_resident_worker'),
    '--resident-ttl-ms','15000','--resident-frame-budget','3',
    '--timeout-ms','15000','--max-requests','1']
  p=subprocess.Popen(cmd,stdin=subprocess.DEVNULL,
      stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
  observations=[]
  try:
   for _ in range(150):
    if sock.exists():break
    if p.poll() is not None:raise RuntimeError('STARTUP_FAILED '+p.stderr.read()[-400:])
    time.sleep(.03)
   else:raise RuntimeError('SOCKET_NOT_CREATED')
   with socket.socket(socket.AF_UNIX,socket.SOCK_SEQPACKET) as conn:
    conn.settimeout(30);conn.connect(str(sock))
    hello,fds=talk(conn,{'type':'HELLO'})
    assert hello['type']=='HELLO' and not fds
    caps,fds=talk(conn,{'type':'CAPABILITIES'})
    assert caps['type']=='CAPABILITIES' and not fds
    prim=[{'x':0.,'y':0.,'radius':.23,'op':0},
       {'x':.10,'y':.02,'radius':.12,'op':3},
       {'x':-.04,'y':-.09,'radius':.048,'op':1}]
    for backend in ('cuda','vulkan'):
     gen=[];ms=[];hashes=[]
     for i in range(7):
      scene=[dict(x) for x in prim]
      scene[1]['x']=round(.10+.035*i,4)
      t=time.perf_counter_ns()
      meta,pixels=frame(conn,'lit',backend,i+1,320,224,scene,100+i)
      ms.append((time.perf_counter_ns()-t)/1e6)
      gen.append(meta['generation']);hashes.append(sha_bytes(pixels))
     assert len(set(hashes))>=5
     assert gen[0] < gen[3] < gen[6],('BUDGET_RENEWAL_FAILED',backend,gen)
     observations.append({'backend':backend,'latencies_ms':[round(v,2) for v in ms],
       'generations':gen,'first_ms':round(ms[0],2),
       'warm_frame_median_ms':round(statistics.median([ms[1],ms[2],ms[4],ms[5]]),2),
       'distinct_rgba_sha':len(set(hashes)), 'budget_rollover':'PASS'})
     print('RESIDENT_DAEMON_E2E_PASS',observations[-1],flush=True)
     # Regression: exact same geometry is intentionally cached by the real
     # CUDA-04 provider. It MUST NOT kill the resident worker, and the cache
     # provenance must be accurate on FRAME_READY.
     meta_cached,lit_cached=frame(conn,'lit',backend,80,320,224,scene,300+i)
     assert meta_cached['cache_hit'] is True and lit_cached==pixels
     assert meta_cached['geometry_sha256']==meta['geometry_sha256']
     meta_norm,norm_cached=frame(conn,'normals',backend,81,320,224,scene,400+i)
     assert meta_norm['cache_hit'] is True and norm_cached!=lit_cached
     assert meta_norm['geometry_sha256']==meta['geometry_sha256']
     observations[-1]['repeat_geometry_cache']='PASS'
     observations[-1]['mode_switch_recolor_cached']='PASS'
    cpu_meta,cpu_bytes=frame(conn,'lit','cpu',20,96,64,prim,220)
    assert cpu_meta['cache_hit'] is False
    # After CPU and Vulkan changes, CUDA must survive the same-geometry
    # request and correctly transition from a renewed worker to its cache.
    renewed_meta,renewed_rgba=frame(conn,'lit','cuda',90,320,224,scene,501)
    cached_meta,cached_rgba=frame(conn,'lit','cuda',91,320,224,scene,502)
    assert renewed_meta['cache_hit'] is False and cached_meta['cache_hit'] is True
    assert renewed_rgba==cached_rgba
    assert cached_meta['generation']==renewed_meta['generation']
    print('CROSS_BACKEND_REENTRY_CACHE_PASS')
    assert cpu_meta['backend_actual']=='cpu' and len(cpu_bytes)==24576
    assert cpu_meta['qualification']=='CPU_LOCAL_PREVIEW_NOT_GPU_QUALIFIED'
  finally:
   if p.poll() is None:
    try:out,err=p.communicate(timeout=15)
    except subprocess.TimeoutExpired:p.kill();out,err=p.communicate()
   else:out,err=p.communicate()
  assert p.returncode==0,(p.returncode,err[-1000:])
  assert not sock.exists(), 'UNEXPECTED_SOCKET_AFTER_EXIT'
  assert 'BLOB_FRAME_EXIT clean=PASS frames=21' in out,out
  assert err.count('descriptor=residency_opt_in')==20,err[-1600:]
  result={'schema':'blobin.resident-renderd-e2e.v1','status':'PASS',
    'backend_results':observations,'cpu_fallback_label':'CPU_ONLY_EXPLICIT',
    'service_original':'UNTOUCHED','socket':'PRIVATE_TEMPORARY','frames':21,'cache_hit_reuse':'PASS',
    'gate_scope':'CUDA05F_GATE_ON_SESSION_START_AND_ROLLOVER',
    'limitations':['same UID, not independent security sandbox',
      'no test of spontaneous GPU reset','no native Wayland window in this test']}
  if args.receipt:
   args.receipt.parent.mkdir(parents=True,exist_ok=True)
   args.receipt.write_text(json.dumps(result,indent=2)+'\n')
  print('RESIDENT_RENDERD_OPTIN_ALL_GATES_PASS')
def sha_bytes(b):return hashlib.sha256(b).hexdigest()
if __name__=='__main__':main()
