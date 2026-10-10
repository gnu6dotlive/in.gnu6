#!/usr/bin/env python3
"""Zero-external-dependency BLOB-FRAME-v1 probe for Claude's native viewer integration.

Queries the REAL local Rust bridge, validates a single sealed read-only memfd,
optionally emits a simple PNG for manual review. Does NOT simulate rendering.
"""
import argparse
import array
import fcntl
import hashlib
import json
import os
from pathlib import Path
import socket
import struct
import sys
import zlib

VERSION='BLOB-FRAME-v1'
SEALS=fcntl.F_SEAL_SEAL|fcntl.F_SEAL_WRITE|fcntl.F_SEAL_GROW|fcntl.F_SEAL_SHRINK

def send(sock, message):
    packet=json.dumps({'version':VERSION,**message},separators=(',',':')).encode()
    if len(packet)>16384:raise ValueError('LOCAL_CONTROL_TOO_LONG')
    sock.send(packet)
    raw,control,flags,_=sock.recvmsg(16384,socket.CMSG_SPACE(4),getattr(socket,'MSG_CMSG_CLOEXEC',0))
    if not raw or flags & (socket.MSG_TRUNC|socket.MSG_CTRUNC):raise ValueError('RESPONSE_TRUNCATED')
    response=json.loads(raw)
    fds=[]
    for level,kind,value in control:
        if (level,kind)==(socket.SOL_SOCKET,socket.SCM_RIGHTS):
            data=array.array('i');data.frombytes(value[:len(value)-len(value)%data.itemsize]);fds.extend(data)
    return response,fds

def write_png(out,width,height,pixels):
    # 8-bit RGBA, scanlines with filter 0. No Pillow required.
    def chunk(label,data):return struct.pack('>I',len(data))+label+data+struct.pack('>I',zlib.crc32(label+data)&0xffffffff)
    scanline=bytearray()
    stride=width*4
    for y in range(height):
        scanline.append(0)
        scanline.extend(pixels[y*stride:(y+1)*stride])
    encoded=(b'\x89PNG\r\n\x1a\n'+chunk(b'IHDR',struct.pack('>2I5B',width,height,8,6,0,0,0))+
             chunk(b'IDAT',zlib.compress(bytes(scanline),6))+chunk(b'IEND',b''))
    out.parent.mkdir(parents=True,exist_ok=True)
    out.write_bytes(encoded)

SCENES={
 'fusion':[
  {'x':-.12,'y':.03,'radius':.18,'op':0},
  {'x':.13,'y':-.03,'radius':.16,'op':3},
  {'x':0.03,'y':-.11,'radius':.045,'op':1}],
 'cutouts':[
  {'x':0.,'y':0.,'radius':.27,'op':0},
  {'x':.03,'y':0.,'radius':.13,'op':1},
  {'x':-.18,'y':-.05,'radius':.10,'op':2},
  {'x':.24,'y':.08,'radius':.12,'op':0}],
 'organic':[
  {'x':0.,'y':0.,'radius':.14,'op':0},
  {'x':-.14,'y':-.05,'radius':.11,'op':3},
  {'x':.13,'y':.07,'radius':.12,'op':3},
  {'x':-.025,'y':.14,'radius':.10,'op':3},
  {'x':.08,'y':-.14,'radius':.11,'op':3}],
}

def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--socket',type=Path,default=Path(os.environ.get('XDG_RUNTIME_DIR','/run/user/'+str(os.getuid())))/'blobin-showcase/render.sock')
    p.add_argument('--backend',choices=['cpu','cuda','vulkan'],default='cuda')
    p.add_argument('--mode',choices=['lit','normals','distance','coverage'],default='lit')
    p.add_argument('--scene',choices=SCENES.keys(),default='fusion')
    p.add_argument('--width',type=int,default=512)
    p.add_argument('--height',type=int,default=384)
    p.add_argument('--out',type=Path)
    a=p.parse_args()
    with socket.socket(socket.AF_UNIX,socket.SOCK_SEQPACKET) as s:
        s.settimeout(60);s.connect(str(a.socket))
        hello,fds=send(s,{'type':'HELLO'})
        assert hello['type']=='HELLO' and hello['version']==VERSION and not fds
        capabilities,fds=send(s,{'type':'CAPABILITIES'})
        assert capabilities['type']=='CAPABILITIES' and not fds
        reply,fds=send(s,{'type':'RENDER_REQUEST','request_id':1,'scene_revision':1,
           'width_px':a.width,'height_px':a.height,'backend_requested':a.backend,
           'view_mode':a.mode,'primitives':SCENES[a.scene]})
        if reply['type']!='FRAME_READY':
            for fd in fds:os.close(fd)
            raise ValueError('REAL_FRAME_NOT_AVAILABLE '+str(reply))
        assert len(fds)==1 and reply['backend_actual']==a.backend
        assert reply['request_id']==reply['scene_revision']==1
        assert reply['format']=='RGBA8_UNORM_OPAQUE' and reply['stride_bytes']==a.width*4
        assert reply['buffer_bytes']==a.width*a.height*4
        fd=fds[0]
        try:
            assert os.fstat(fd).st_size==reply['buffer_bytes']
            assert fcntl.fcntl(fd,fcntl.F_GET_SEALS)&SEALS==SEALS
            with os.fdopen(os.dup(fd),'rb') as file:
                pixels=file.read()
            assert len(pixels)==reply['buffer_bytes']
            assert all(x==255 for x in pixels[3::4])
        finally:
            os.close(fd)
    print('BLOB_FRAME_REAL_RENDER_PASS',json.dumps({
        'backend_actual':reply['backend_actual'],'view_mode':reply['view_mode'],
        'scene_revision':reply['scene_revision'],'generation':reply['generation'],
        'qualification':reply['qualification'],'dimensions':[a.width,a.height],
        'rgba_sha256':hashlib.sha256(pixels).hexdigest(),
        'geometry_sha256':reply['geometry_sha256'],'sealed_fd':True},sort_keys=True))
    if a.out:
        write_png(a.out,a.width,a.height,pixels)
        print('REAL_COMPUTE_PNG_WRITTEN',a.out)
if __name__=='__main__':
    try:main()
    except (ValueError,AssertionError,OSError) as e:sys.exit('BLOB_FRAME_PROBE_FAIL '+str(e))
