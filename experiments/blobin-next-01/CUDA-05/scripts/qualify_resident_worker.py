#!/usr/bin/env python3
"""Qualification of a bounded GPU-resident worker (NOT yet integrated with preview daemon)."""
import argparse, hashlib, json, os, pathlib, struct, subprocess, time, statistics
def sha(path): return hashlib.sha256(path.read_bytes()).hexdigest()
def spawn(binary,backend,lib,frames=48,ttl=60000,hash_override=None):
    libname=str(lib) if backend=="cuda" else "-"
    digest=(hash_override if hash_override is not None else sha(lib)) if backend=="cuda" else "-"
    return subprocess.Popen([str(binary),backend,libname,digest,str(frames),str(ttl)],
        stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
def send(p,request):
    data=json.dumps({"version":"BLOB-FRAME-v1",**request},separators=(',',':')).encode()
    p.stdin.write(struct.pack("<I",len(data))+data); p.stdin.flush()
    prefix=p.stdout.read(4)
    if len(prefix)!=4:raise RuntimeError("RESPONSE_HEADER_MISSING")
    size=struct.unpack("<I",prefix)[0]
    if size != request["width_px"]*request["height_px"]*4:raise RuntimeError("SIZE_MISMATCH")
    flag=p.stdout.read(1)
    if flag not in (b"\x00",b"\x01"):raise RuntimeError("CACHE_FLAG_INVALID")
    data=p.stdout.read(size)
    if len(data)!=size or data[3::4]!=b"\xff"*(size//4):raise RuntimeError("OUTPUT_INCOMPLETE_OR_ALPHA")
    return data
def scene(step,frames):
    import math
    wave=math.sin(step/frames*2*math.pi)
    return [{"x":round(-0.12+wave*0.05,6),"y":0.02,"radius":0.22,"op":0},
      {"x":round(0.14-wave*0.04,6),"y":0.06,"radius":0.13,"op":3},
      {"x":0.0,"y":round(-0.07+wave*0.04,6),"radius":0.04,"op":1}]
def run(binary,lib,backend,frames):
    p=spawn(binary,backend,lib,frames)
    samples=[];digests=set()
    try:
        for i in range(frames):
            req={"type":"RENDER_REQUEST","request_id":i+1,"scene_revision":i+1,
                "width_px":320,"height_px":224,"backend_requested":backend,
                "view_mode":"lit","primitives":scene(i,frames)}
            t=time.perf_counter_ns()
            raw=send(p,req)
            samples.append((time.perf_counter_ns()-t)/1e6)
            digests.add(hashlib.sha256(raw).hexdigest())
        p.stdin.close()
        rc=p.wait(timeout=8)
        err=p.stderr.read().decode(errors='replace')[-1800:]
        if rc!=0:raise RuntimeError(f"UNEXPECTED_CHILD_EXIT {backend}: {err}")
        if len(digests)<frames//2:raise RuntimeError("ANIMATED_FRAMES_NOT_CHANGING")
        if "RESIDENT_WORKER_BUDGET_REVOKED" not in err:raise RuntimeError("FRAME_BUDGET_NOT_ENFORCED")
        return {"backend":backend,"frames":frames,"distinct_sha256":len(digests),
          "first_ms":round(samples[0],3),"warm_median_ms":round(statistics.median(samples[1:]),3),
          "warm_max_ms":round(max(samples[1:]),3),"budget_revocation":"PASS",
          "raw_frame_hashes":list(sorted(digests))[:3]}
    finally:
        if p.poll() is None:p.kill();p.communicate(timeout=3)
def main():
    ap=argparse.ArgumentParser()
    ap.add_argument("--binary",type=pathlib.Path,required=True)
    ap.add_argument("--cuda-lib",type=pathlib.Path,required=True)
    ap.add_argument("--out",type=pathlib.Path)
    args=ap.parse_args()
    records=[run(args.binary,args.cuda_lib,b,48) for b in ("cuda","vulkan")]
    for r in records:print("RESIDENT_48_FRAME_PASS",r,flush=True)
    bad=spawn(args.binary,"cuda",args.cuda_lib,frames=1,hash_override="0"*64)
    bad.stdin.close();rc=bad.wait(timeout=6)
    assert rc!=0 and b"CUDA_PROVIDER_SHA_MISMATCH" in bad.stderr.read(),"MISMATCH_NOT_REJECTED"
    short=spawn(args.binary,"vulkan",args.cuda_lib,frames=3,ttl=1000)
    time.sleep(1.15)
    try:
        send(short,{"type":"RENDER_REQUEST","request_id":9,"scene_revision":1,
         "width_px":96,"height_px":64,"backend_requested":"vulkan",
         "view_mode":"distance","primitives":scene(0,48)})
    except (BrokenPipeError,RuntimeError):pass
    else:raise RuntimeError("EXPIRED_LEASE_REQUEST_ACCEPTED")
    short.stdin.close()
    assert short.wait(timeout=5)!=0 and b"SESSION_TTL_EXPIRED_REVOKED" in short.stderr.read()
    result={"schema":"blobin.resident-worker-p01.v1","workload":"animated SDF CSG 320x224",
      "gpu":"RTX3070","records":records,"bad_cuda_hash":"REJECTED",
      "ttl_expiry":"REJECTED","note":"This directly tests child workers, not the 05F supervisor or Wayland."}
    if args.out:args.out.parent.mkdir(parents=True,exist_ok=True);args.out.write_text(json.dumps(result,indent=2)+"\n")
    print("RESIDENT_CHILD_GATES_PASS backends=2 frame_budget=PASS ttl=PASS hash=PASS")
if __name__=="__main__":main()
