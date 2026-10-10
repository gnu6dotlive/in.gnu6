#!/usr/bin/env python3
"""Reproducible canonical glTF 2.0 tetra geometry fixtures (only stdlib)."""
from pathlib import Path
import hashlib, json, struct
root=Path(__file__).resolve().parents[1]/"fixtures"
root.mkdir(exist_ok=True)
vertices=((0.,0.,0.),(1.,0.,0.),(0.,1.,0.),(0.,0.,1.))
indices=(0,2,1,0,1,3,0,3,2,1,2,3)
body=b"".join(struct.pack("<fff",*v) for v in vertices)+struct.pack("<12H",*indices)
assert len(body)==72
doc={
 "asset":{"version":"2.0","generator":"BLOB-NEXT-01-GEO03-fixture-generator"},
 "buffers":[{"byteLength":len(body)}],
 "bufferViews":[{"buffer":0,"byteOffset":0,"byteLength":48},
                {"buffer":0,"byteOffset":48,"byteLength":24}],
 "accessors":[{"bufferView":0,"componentType":5126,"count":4,"type":"VEC3","min":[0,0,0],"max":[1,1,1]},
              {"bufferView":1,"componentType":5123,"count":12,"type":"SCALAR"}],
 "meshes":[{"primitives":[{"attributes":{"POSITION":0},"indices":1,"mode":4}]}],
 "nodes":[{"mesh":0}],"scenes":[{"nodes":[0]}],"scene":0
}
j=json.dumps(doc,sort_keys=True,separators=(",",":")).encode()
(doc_glb:=dict(doc))["buffers"]=[{"byteLength":len(body)}]
glb_json=json.dumps(doc_glb,sort_keys=True,separators=(",",":")).encode()
glb_json+=b" " * (-len(glb_json)%4)
out=b"glTF"+struct.pack("<II",2,12+8+len(glb_json)+8+len(body))
out+=struct.pack("<II",len(glb_json),0x4e4f534a)+glb_json
out+=struct.pack("<II",len(body),0x004e4942)+body
doc["buffers"]=[{"byteLength":len(body),"uri":"tetra.bin"}]
gltf=(json.dumps(doc,sort_keys=True,indent=2)+"\n").encode()
for filename,content in [("tetra-indexed.glb",out),("tetra.gltf",gltf),("tetra.bin",body)]:
    path=root/filename
    path.write_bytes(content)
    print(filename,len(content),hashlib.sha256(content).hexdigest())

# GEO-04: independent triangulated concave L-shape, deliberately not a
# generalized arbitrary mesh-solid certificate.
contour=((0.,0.),(3.,0.),(3.,1.),(1.,1.),(1.,3.),(0.,3.))
vertices=tuple((x,y,z) for z in (-1.,1.) for x,y in contour)
cap=((0,1,3),(1,2,3),(0,3,5),(3,4,5))
triangles=[]
for a,b,c in cap:
    triangles.extend(((c,b,a),(a+6,b+6,c+6)))
for i in range(6):
    j=(i+1)%6
    triangles.extend(((i,j,j+6),(i,j+6,i+6)))
body=b"".join(struct.pack("<fff",*v) for v in vertices)
positions_len=len(body)
indices=tuple(n for t in triangles for n in t)
body+=struct.pack(f"<{len(indices)}H",*indices)
assert len(body)%4==0
doc={
 "asset":{"version":"2.0","generator":"BLOB-NEXT-01-GEO04-l-prism-generator"},
 "buffers":[{"byteLength":len(body)}],
 "bufferViews":[{"buffer":0,"byteOffset":0,"byteLength":positions_len},
                {"buffer":0,"byteOffset":positions_len,"byteLength":len(indices)*2}],
 "accessors":[{"bufferView":0,"componentType":5126,"count":len(vertices),"type":"VEC3"},
              {"bufferView":1,"componentType":5123,"count":len(indices),"type":"SCALAR"}],
 "meshes":[{"primitives":[{"attributes":{"POSITION":0},"indices":1,"mode":4}]}],
 "nodes":[{"mesh":0}],"scenes":[{"nodes":[0]}],"scene":0
}
j=json.dumps(doc,sort_keys=True,separators=(",",":")).encode()
j+=b" " * (-len(j)%4)
out=b"glTF"+struct.pack("<II",2,12+8+len(j)+8+len(body))
out+=struct.pack("<II",len(j),0x4e4f534a)+j
out+=struct.pack("<II",len(body),0x004e4942)+body
path=root/"l-prism-nonconvex.glb"
path.write_bytes(out)
print(path.name,len(out),hashlib.sha256(out).hexdigest())
