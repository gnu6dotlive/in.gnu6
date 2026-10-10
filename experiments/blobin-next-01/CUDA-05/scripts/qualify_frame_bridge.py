#!/usr/bin/env python3
"""Offline black-box BLOB-FRAME-v1 qualification against real local CUDA-05F gate.

Requires built Rust binaries and native CUDA-03 provider. Nothing is published or
copied to the viewer project; receipts are stored only where the operator requests.
"""
import argparse
import array
import fcntl
import hashlib
import json
import os
from pathlib import Path
import socket
import subprocess
import tempfile
import time

SEALS=fcntl.F_SEAL_SEAL|fcntl.F_SEAL_WRITE|fcntl.F_SEAL_GROW|fcntl.F_SEAL_SHRINK

def digest(path): return hashlib.sha256(path.read_bytes()).hexdigest()

def talk(s,message):
    packet=json.dumps({'version':'BLOB-FRAME-v1',**message},separators=(',',':')).encode()
    assert len(packet)<16384
    s.send(packet)
    data,control,flags,_=s.recvmsg(16384,socket.CMSG_SPACE(4),getattr(socket,'MSG_CMSG_CLOEXEC',0))
    assert data and not (flags&(socket.MSG_TRUNC|socket.MSG_CTRUNC)), 'malformed packet'
    obj=json.loads(data)
    fds=[]
    for level,kind,body in control:
        if (level,kind)==(socket.SOL_SOCKET,socket.SCM_RIGHTS):
            a=array.array('i');a.frombytes(body[:len(body)-(len(body)%a.itemsize)]);fds.extend(a)
    return obj,fds

def frame(s,mode,backend,revision,width,height,scene,id):
    payload={'type':'RENDER_REQUEST','request_id':id,'scene_revision':revision,
        'backend_requested':backend,'view_mode':mode,'width_px':width,'height_px':height,
        'primitives':scene}
    meta,fds=talk(s,payload)
    assert meta['type']=='FRAME_READY',(meta,backend,mode)
    assert meta['request_id']==id and meta['scene_revision']==revision
    assert meta['backend_requested']==backend and meta['backend_actual']==backend
    assert meta['view_mode']==mode and meta['format']=='RGBA8_UNORM_OPAQUE'
    assert meta['stride_bytes']==width*4 and meta['buffer_bytes']==width*height*4
    assert len(fds)==1, 'one and only one SCM_RIGHTS descriptor'
    fd=fds[0]
    try:
        assert os.fstat(fd).st_size==width*height*4
        assert (fcntl.fcntl(fd,fcntl.F_GET_SEALS)&SEALS)==SEALS
        with os.fdopen(os.dup(fd),'rb') as reader:payload=reader.read()
        assert len(payload)==width*height*4,'readable fd must be rewound before send'
        assert payload[3::4]==bytes([255])*(width*height)
        try:
            os.write(fd,b'\xff')
        except OSError:pass
        else:raise AssertionError('received memfd remains writable')
    finally:os.close(fd)
    return meta,payload

def main():
    p=argparse.ArgumentParser()
    p.add_argument('--bin-dir',type=Path,required=True)
    p.add_argument('--cuda-library',type=Path,required=True)
    p.add_argument('--receipt',type=Path)
    args=p.parse_args()
    daemon=args.bin_dir/'blobin_renderd'
    worker=args.bin_dir/'blobin_frame_worker'
    gate=args.bin_dir/'worker05c'
    lib=args.cuda_library
    for x in (daemon,worker,gate,lib):assert x.is_file(),f'REQUIRED_ARTIFACT_MISSING {x}'
    evidence={'schema':'gnu6.blobin.frame-bridge-smoke.v1','scope':'local-preview-real-provider',
        'worker_sha256':digest(worker),'gate_sha256':digest(gate),
        'cuda_provider_sha256':digest(lib),'tests':{},'boundary':[
        'same desktop UID only','not separate hostile UID containment',
        'CUDA-05F static probe precedes frame worker execution',
        'no native Wayland window qualified here']}
    with tempfile.TemporaryDirectory(prefix='blobin-fv1-') as folder:
        path=Path(folder)/'s.sock'
        argv=[str(daemon),'--socket',str(path),
              '--frame-worker',str(worker),'--frame-sha256',digest(worker),
              '--gate-worker',str(gate),'--gate-sha256',digest(gate),
              '--cuda-lib',str(lib),'--cuda-sha256',digest(lib),
              '--timeout-ms','15000','--max-requests','1']
        server=subprocess.Popen(argv,stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
        try:
            for _ in range(130):
                if path.exists():break
                if server.poll() is not None:raise RuntimeError('PREMATURE_EXIT '+server.stderr.read()[-1000:])
                time.sleep(.03)
            else:raise RuntimeError('STARTUP_TIMEOUT')
            with socket.socket(socket.AF_UNIX,socket.SOCK_SEQPACKET) as s:
                s.settimeout(60);s.connect(str(path))
                for kind in ('HELLO','CAPABILITIES','PING'):
                    answer,fds=talk(s,{'type':kind});assert not fds
                    assert answer['type']==('PONG' if kind=='PING' else kind)
                evidence['tests']['handshake']='PASS'
                for mutate in (
                    {'type':'RENDER_REQUEST','width_px':100000},
                    {'type':'RENDER_REQUEST','request_id':1,'scene_revision':1,'width_px':96,
                     'height_px':64,'backend_requested':'auto','view_mode':'lit',
                     'primitives':[{'x':0,'y':0,'radius':.3,'op':0}]},
                    {'type':'RENDER_REQUEST','request_id':1,'scene_revision':1,'width_px':96,
                     'height_px':64,'backend_requested':'cuda','view_mode':'lit',
                     'primitives':[{'x':0,'y':0,'radius':.3,'op':99}]},
                ):
                    response,fds=talk(s,mutate)
                    assert response['type']=='RENDER_ERROR' and response['reason']=='INVALID_CONTROL_MESSAGE' and not fds
                evidence['tests']['strict_request_validation']='PASS'
                shapes=[{'x':0,'y':0,'radius':.23,'op':0},
                        {'x':.09,'y':.05,'radius':.135,'op':3},
                        {'x':-.08,'y':-.045,'radius':.062,'op':1}]
                references={}
                for i,backend in enumerate(('cpu','cuda','vulkan')):
                    meta,data=frame(s,'lit',backend,7,96,64,shapes,10+i)
                    references[backend]=data
                    if backend=='cpu':assert meta['qualification']=='CPU_LOCAL_PREVIEW_NOT_GPU_QUALIFIED'
                    else:assert meta['qualification']=='CUDA05F_GATE_PASS'
                    evidence['tests'][f'backend_{backend}']='PASS'
                assert references['cpu']==references['cuda']==references['vulkan']
                evidence['tests']['rgba_exact_3_backend_parity']='PASS'
                for i,mode in enumerate(('normals','distance','coverage')):
                    meta,data=frame(s,mode,'cuda',8,96,64,shapes,20+i)
                    assert len(data)==24576
                    evidence['tests'][f'view_{mode}']='PASS'
                modified=[dict(p) for p in shapes];modified[1]['radius']=.16
                meta,data=frame(s,'lit','cuda',9,128,80,modified,30)
                assert meta['scene_revision']==9 and len(data)==40960
                evidence['tests']['scene_revision_and_resize']='PASS'
            stdout,stderr=server.communicate(timeout=40)
            assert server.returncode==0,(server.returncode,stderr[-1200:])
            assert 'BLOB_FRAME_EXIT clean=PASS frames=7' in stdout,(stdout,stderr)
            assert not path.exists(),'STALE_SOCKET_AFTER_CLEAN_EXIT'
            evidence['tests']['service_exit_and_socket_cleanup']='PASS'
            # Negative executable pin: startup must reject before binding a socket.
            wrong=argv.copy()
            wrong[wrong.index('--frame-sha256')+1]='0'*64
            wrong[wrong.index('--socket')+1]=str(Path(folder)/'tampered.sock')
            denied=subprocess.run(wrong,stdin=subprocess.DEVNULL,text=True,
               capture_output=True,timeout=5)
            assert denied.returncode!=0 and 'APPROVED_EXECUTABLE_CHANGED' in denied.stderr
            assert not (Path(folder)/'tampered.sock').exists()
            evidence['tests']['executable_digest_fails_closed']='PASS'
        finally:
            if server.poll() is None:
                server.kill();server.communicate()
    if args.receipt:
        args.receipt.parent.mkdir(parents=True,exist_ok=True)
        args.receipt.write_text(json.dumps(evidence,sort_keys=True,indent=2)+'\n')
    assert all(v=='PASS' for v in evidence['tests'].values())
    print('BLOB_FRAME_BLACKBOX_PASS',len(evidence['tests']),
          'gates gpu_backends=2 scene_revision=PASS resized=PASS sealed_fd=PASS worker_hash=FAIL_CLOSED')
if __name__=='__main__':main()
