// GPU-01 Vulkan: raymarch a REAL Euclidean signed distance to the faces
// of a convex certified imported triangle MeshSolid; subtract an SDF sphere.
// Compute-only offscreen preview; does NOT present frames through Wayland.
struct Mesh { triangles:array<vec4<f32>,12>, planes:array<vec4<f32>,4>, }
struct Samples { values:array<vec4<f32>>, }
struct Pixels { values:array<u32>, }
@group(0) @binding(0) var<storage,read> mesh:Mesh;
@group(0) @binding(1) var<storage,read> samples:Samples;
@group(0) @binding(2) var<storage,read_write> sample_out:Samples;
@group(0) @binding(3) var<storage,read_write> output:Pixels;
struct Animation { time:f32, yaw:f32, sphere_offset_x:f32, camera_time:f32, }
@group(0) @binding(4) var<uniform> animation:Animation;

fn triangle_distance(p:vec3<f32>,a:vec3<f32>,b:vec3<f32>,c:vec3<f32>)->f32 {
 let ab=b-a;let ac=c-a;let ap=p-a;
 let d1=dot(ab,ap);let d2=dot(ac,ap);
 if d1<=0.0 && d2<=0.0 {return distance(p,a);}
 let bp=p-b;let d3=dot(ab,bp);let d4=dot(ac,bp);
 if d3>=0.0 && d4<=d3 {return distance(p,b);}
 let vc=d1*d4-d3*d2;
 if vc<=0.0 && d1>=0.0 && d3<=0.0 {
   let v=d1/(d1-d3);return distance(p,a+ab*v);
 }
 let cp=p-c;let d5=dot(ab,cp);let d6=dot(ac,cp);
 if d6>=0.0 && d5<=d6 {return distance(p,c);}
 let vb=d5*d2-d1*d6;
 if vb<=0.0 && d2>=0.0 && d6<=0.0 {
   let w=d2/(d2-d6);return distance(p,a+ac*w);
 }
 let va=d3*d6-d5*d4;
 if va<=0.0 && (d4-d3)>=0.0 && (d5-d6)>=0.0 {
   let w=(d4-d3)/((d4-d3)+(d5-d6));return distance(p,b+(c-b)*w);
 }
 let denom=1.0/(va+vb+vc);
 let v=vb*denom;let w=vc*denom;
 return distance(p,a+ab*v+ac*w);
}
fn tetra_signed(p:vec3<f32>)->f32 {
 var near=1e30;var exterior=-1e30;
 for(var i:u32=0u;i<4u;i=i+1u) {
   let a=mesh.triangles[i*3u].xyz;
   let b=mesh.triangles[i*3u+1u].xyz;
   let c=mesh.triangles[i*3u+2u].xyz;
   near=min(near,triangle_distance(p,a,b,c));
   let pl=mesh.planes[i];
   exterior=max(exterior,dot(pl.xyz,p)+pl.w);
 }
 if exterior>0.0{return near;}
 return -near;
}
fn sphere_signed(p:vec3<f32>)->f32 {
 return distance(p,vec3<f32>(0.42+animation.sphere_offset_x,0.34,0.47))-0.38;
}
fn hybrid(p:vec3<f32>)->f32 {
 return max(tetra_signed(p),-sphere_signed(p));
}
fn pack_rgb(rgb:vec3<f32>)->u32 {
 let clamped=clamp(rgb,vec3<f32>(0.0),vec3<f32>(1.0));
 let r=u32(clamped.r*255.0+0.5);
 let g=u32(clamped.g*255.0+0.5);
 let b=u32(clamped.b*255.0+0.5);
 return r | (g<<8u) | (b<<16u) | (255u<<24u);
}
@compute @workgroup_size(64)
fn field_samples(@builtin(global_invocation_id) id:vec3<u32>) {
 let i=id.x;
 if i>=arrayLength(&samples.values){return;}
 let p=samples.values[i].xyz;
 sample_out.values[i]=vec4<f32>(tetra_signed(p),sphere_signed(p),hybrid(p),0.0);
}
@compute @workgroup_size(8,8,1)
fn render(@builtin(global_invocation_id) id:vec3<u32>) {
 let w:u32=640u;let h:u32=432u;
 if id.x>=w || id.y>=h{return;}
 let uv=(vec2<f32>(f32(id.x)+0.5,f32(id.y)+0.5)/vec2<f32>(f32(w),f32(h)))*2.0-vec2<f32>(1.0);
 let eye=vec3<f32>(0.24+2.1*cos(0.78+animation.yaw+animation.camera_time*0.23),1.6+0.12*sin(animation.camera_time*0.19),0.24+2.1*sin(0.78+animation.yaw+animation.camera_time*0.23));
 let focus=vec3<f32>(0.24,0.23,0.24);
 let forward=normalize(focus-eye);
 let right=normalize(cross(forward,vec3<f32>(0.0,1.0,0.0)));
 let up=cross(right,forward);
 let rd=normalize(forward+right*uv.x*0.46+up*(-uv.y)*0.32);
 let bg=mix(vec3<f32>(0.075,0.105,0.135),vec3<f32>(0.018,0.028,0.046),clamp((uv.y+1.0)*0.5,0.0,1.0));
 var color=bg;
 // ground plane and fine world-grid, separate from geometry hit.
 if rd.y< -0.0001{
    let ground_t=(-0.10-eye.y)/rd.y;
    if ground_t>0.0 && ground_t<8.0{
      let gp=eye+rd*ground_t;
      let line_x=abs(fract(gp.x*4.0)-0.5);
      let line_z=abs(fract(gp.z*4.0)-0.5);
      let line=select(0.0,1.0,min(line_x,line_z)>0.485);
      let fade=exp(-0.13*ground_t*ground_t);
      color=mix(bg,vec3<f32>(0.09,0.14,0.17)+vec3<f32>(0.07,0.11,0.12)*line,0.58*fade);
    }
 }
 var t=0.0;var found=false;
 for(var i:u32=0u;i<84u;i=i+1u){
   let p=eye+rd*t;
   let d=hybrid(p);
   if abs(d)<0.0012{found=true;break;}
   t=t+max(abs(d)*0.72,0.002);
   if t>6.5{break;}
 }
 if found{
   let p=eye+rd*t;
   let e=0.0025;
   let nx=hybrid(p+vec3<f32>(e,0.0,0.0))-hybrid(p-vec3<f32>(e,0.0,0.0));
   let ny=hybrid(p+vec3<f32>(0.0,e,0.0))-hybrid(p-vec3<f32>(0.0,e,0.0));
   let nz=hybrid(p+vec3<f32>(0.0,0.0,e))-hybrid(p-vec3<f32>(0.0,0.0,e));
   let n=normalize(vec3<f32>(nx,ny,nz));
   let light=normalize(vec3<f32>(-0.30,1.10,0.80));
   let key=max(dot(n,light),0.0);
   let fill=max(dot(n,normalize(vec3<f32>(0.8,0.2,-0.8))),0.0)*0.32;
   let spec=pow(max(dot(reflect(-light,n),-rd),0.0),24.0);
   let cut=(-sphere_signed(p)) > tetra_signed(p);
   let base=select(vec3<f32>(0.97,0.43,0.19),vec3<f32>(0.12,0.71,0.78),cut);
   let shade=0.32+0.62*key+fill;
   let fog=exp(-0.09*t*t);
   let rim=pow(1.0-max(dot(n,-rd),0.0),3.0);
   color=mix(bg,base*shade+vec3<f32>(0.20)*spec +vec3<f32>(0.16,0.23,0.28)*rim,fog);
 }
 // linear to gamma for the preview only
 color=pow(max(color,vec3<f32>(0.0)),vec3<f32>(0.85));
 output.values[id.y*w+id.x]=pack_rgb(color);
}
