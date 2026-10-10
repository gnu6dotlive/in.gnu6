#!/usr/bin/env python3
"""Reproduce reported high-res CUDA→Vulkan→CPU→CUDA crash; require correct cache provenance."""
import argparse
import hashlib
import json
from pathlib import Path
import socket
import subprocess
import tempfile
import time
from qualify_frame_bridge import frame, talk

def sha(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
    ap=argparse.ArgumentParser()
    ap.add_argument('--bin-dir',type=Path,required=True)
    ap.add_argument('--cuda-library',type=Path,required=True)
    ap.add_argument('--receipt',type=Path)
    a=ap.parse_args()
    b=a.bin_dir
    with tempfile.TemporaryDirectory(prefix='blobin-real-resize-') as tmp:
        path=Path(tmp)/'r.sock'
        args=[str(b/'blobin_renderd'),'--socket',str(path),
            '--frame-worker',str(b/'blobin_frame_worker'),
            '--frame-sha256',sha(b/'blobin_frame_worker'),
            '--gate-worker',str(b/'worker05c'),
            '--gate-sha256',sha(b/'worker05c'),
            '--cuda-lib',str(a.cuda_library),
            '--cuda-sha256',sha(a.cuda_library),
            '--resident-worker',str(b/'blobin_resident_worker'),
            '--resident-sha256',sha(b/'blobin_resident_worker'),
            '--resident-ttl-ms','30000',
            '--resident-frame-budget','120',
            '--timeout-ms','15000',
            '--max-requests','1']
        proc=subprocess.Popen(args,stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
        rows=[]
        try:
            for i in range(150):
                if path.exists():break
                if proc.poll() is not None:raise RuntimeError('PREMATURE_EXIT '+proc.stderr.read()[-500:])
                time.sleep(.03)
            else:raise RuntimeError('LISTENER_NOT_READY')
            with socket.socket(socket.AF_UNIX,socket.SOCK_SEQPACKET) as s:
                s.settimeout(22)
                s.connect(str(path))
                result,fd=talk(s,{'type':'HELLO'})
                assert result['type']=='HELLO' and not fd
                shapes=[{'x':0.,'y':0.,'radius':.23,'op':0},
                        {'x':.11,'y':.04,'radius':.12,'op':3},
                        {'x':-.06,'y':-.04,'radius':.045,'op':1}]
                sequence=[('cuda',210,748),('cuda',205,737),('cuda',204,733),
                    ('cuda',372,733),('cuda',642,1024),('vulkan',642,1024),
                    ('vulkan',642,1024),('cpu',642,1024),
                    ('cuda',642,1024),('cuda',642,1024)]
                last={}
                for number,(backend,w,h) in enumerate(sequence):
                    started=time.monotonic()
                    meta,pixels=frame(s,'lit',backend,number+1,w,h,shapes,number+1)
                    digest=hashlib.sha256(pixels).hexdigest()
                    same=(backend,w,h) in last
                    assert meta['cache_hit'] is same, (backend,number,meta.get('cache_hit'),same)
                    if same:
                        assert digest==last[(backend,w,h)], ('CACHE_FRAME_CHANGED',backend,number)
                    else:
                        last[(backend,w,h)]=digest
                    row={'index':number,'backend':backend,'dimensions':[w,h],
                         'roundtrip_ms':round((time.monotonic()-started)*1000,2),
                         'cache_hit':meta['cache_hit'],
                         'generation':meta['generation'],'rgba_sha256':digest}
                    rows.append(row)
                    print('HIGH_RES_SWITCH_PASS',row,flush=True)
            out,err=proc.communicate(timeout=25)
            assert proc.returncode==0,(proc.returncode,err[-700:])
            assert 'BLOB_FRAME_EXIT clean=PASS frames=10' in out
            assert not path.exists()
            assert 'FRAME_RENDER_REJECTED' not in err,err[-1000:]
            result={'schema':'blobin.resident-cache-resize-regression.v1',
                 'sequence':'cuda_5_vulkan_2_cpu_1_cuda_2','frames':rows,
                 'cache_hit_checks':'PASS',
                 'repeated_gpu_geometry_no_crash':'PASS',
                 'renderer_exit':'CLEAN',
                 'scope':'test client on private temporary socket, not Wayland UI'}
            if a.receipt:
                a.receipt.parent.mkdir(parents=True,exist_ok=True)
                a.receipt.write_text(json.dumps(result,indent=2)+'\n')
            print('BLOB_CACHE_RESIZE_SWITCH_PASS frames=10 no_false_timeout cache_provenance=PASS')
        finally:
            if proc.poll() is None:
                proc.kill();proc.communicate()
if __name__=='__main__':main()
