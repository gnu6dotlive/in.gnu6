#!/usr/bin/env python3
"""CPU/CUDA/Vulkan staged performance profile against actual native Render Bridge.

Launches one ephemeral server using an isolated AF_UNIX socket, makes a small finite
set of 2D SDF renders, and captures instrumented gate / worker durations. Neither
FPS estimates nor timings are claimed as hard GPU kernel-only measurements.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import socket
import statistics
import subprocess
import tempfile
import time
from qualify_frame_bridge import talk,frame

TIMING=re.compile(r'BLOB_FRAME_TIMING backend=(\w+) mode=(\w+) width=(\d+) height=(\d+) gate_ms=([0-9.]+) worker_ms=([0-9.]+) total_ms=([0-9.]+) sequence=(\d+)')

def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def stats(values):
    values=sorted(values)
    if not values:return None
    return {'n':len(values),'min_ms':round(values[0],3),'median_ms':round(statistics.median(values),3),
            'p95_ms':round(values[min(len(values)-1,max(0,int(len(values)*.95+.999)-1))],3),
            'max_ms':round(values[-1],3)}
def main():
    ap=argparse.ArgumentParser()
    ap.add_argument('--bin-dir',type=Path,required=True)
    ap.add_argument('--cuda-library',type=Path,required=True)
    ap.add_argument('--out',type=Path)
    ap.add_argument('--iterations',type=int,default=4)
    a=ap.parse_args()
    if not 2<=a.iterations<=12:raise ValueError('iterations must be 2..12')
    bin=a.bin_dir;lib=a.cuda_library
    for file in (bin/'blobin_renderd',bin/'blobin_frame_worker',bin/'worker05c',lib):
        if not file.is_file():raise ValueError('REQUIRED_BINARY_MISSING '+str(file))
    with tempfile.TemporaryDirectory(prefix='blobin-prof-') as folder:
        sock_path=Path(folder)/'s.sock'
        cmd=[str(bin/'blobin_renderd'),'--socket',str(sock_path),
             '--frame-worker',str(bin/'blobin_frame_worker'),'--frame-sha256',sha(bin/'blobin_frame_worker'),
             '--gate-worker',str(bin/'worker05c'),'--gate-sha256',sha(bin/'worker05c'),
             '--cuda-lib',str(lib),'--cuda-sha256',sha(lib),
             '--timeout-ms','15000','--max-requests','1']
        server=subprocess.Popen(cmd,stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
        wall=[]
        try:
            for _ in range(100):
                if sock_path.exists():break
                if server.poll() is not None:raise RuntimeError('DAEMON_EARLY_EXIT '+server.stderr.read()[-300:])
                time.sleep(.04)
            else:raise RuntimeError('SOCKET_TIMEOUT')
            with socket.socket(socket.AF_UNIX,socket.SOCK_SEQPACKET) as sock:
                sock.settimeout(30);sock.connect(str(sock_path))
                hello,fd=talk(sock,{'type':'HELLO'})
                assert hello['type']=='HELLO' and not fd
                prim=[{'x':0.,'y':0.,'radius':.22,'op':0},
                      {'x':.11,'y':.04,'radius':.12,'op':3},
                      {'x':-.06,'y':-.04,'radius':.045,'op':1}]
                sequence=0
                for backend in ('cpu','cuda','vulkan'):
                    for i in range(a.iterations):
                        sequence+=1
                        start=time.monotonic_ns()
                        meta,pixels=frame(sock,'lit',backend,sequence,320,224,prim,sequence)
                        elapsed=(time.monotonic_ns()-start)/1_000_000
                        wall.append({'backend':backend,'iteration':i,'wall_ms':round(elapsed,3),
                          'bytes':len(pixels),'generation':meta['generation']})
            output,error=server.communicate(timeout=30)
            if server.returncode!=0:raise RuntimeError('DAEMON_EXIT '+str(server.returncode)+' '+error[-800:])
        finally:
            if server.poll() is None:
                server.kill();server.communicate()
        observations=[]
        for line in error.splitlines():
            if m:=TIMING.search(line):
                observations.append({'backend':m[1],'mode':m[2],
                    'width':int(m[3]),'height':int(m[4]),'gate_ms':float(m[5]),
                    'worker_ms':float(m[6]),'total_ms':float(m[7]),'sequence':int(m[8])})
        if len(observations)!=len(wall):
            raise AssertionError(f'MISSING_RUNTIME_TIMING {len(observations)} vs {len(wall)}')
        summary={}
        for backend in ('cpu','cuda','vulkan'):
            runtime=[e for e in observations if e['backend']==backend]
            host=[w['wall_ms'] for w in wall if w['backend']==backend]
            summary[backend]={
                'host_roundtrip':stats(host),
                'gate_05f':stats([e['gate_ms'] for e in runtime]),
                'disposable_worker_including_init_compute_readback':stats([e['worker_ms'] for e in runtime]),
                'runtime_before_memfd_transport':stats([e['total_ms'] for e in runtime])}
        result={'schema':'blobin.render-profiler.v1','gpu':'RTX 3070 as reported by host preflight',
            'scene':'3 SDF primitives, 320x224, lit mode','iterations_per_backend':a.iterations,
            'notes':['host wall clocks include IPC','worker wall includes process spawn and GPU initialization',
                     'gate is fixed-scene CUDA-05F qualification, not interactive shader computation',
                     'single host, no claim of standardized benchmarking, no GPU resets'],
            'summary':summary,'samples':{'runtime':observations,'host':wall},
            'daemon_exit':server.returncode,'pass':True}
        if a.out:
            a.out.parent.mkdir(parents=True,exist_ok=True)
            a.out.write_text(json.dumps(result,sort_keys=True,indent=2)+'\n')
        print('BLOB_PROFILE_PASS',json.dumps(summary,sort_keys=True))
if __name__=='__main__':main()
