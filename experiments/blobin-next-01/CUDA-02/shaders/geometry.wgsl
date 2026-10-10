// CUDA-02 common CSG ABI: float4(x,y,r,operation as 0..3), signed field and normal.
struct Uniform { dims:vec4<u32> } // width, height, count, reserved
@group(0) @binding(0) var<uniform> conf:Uniform;
struct Prim { pos_radius:vec4<f32> }
@group(0) @binding(1) var<storage,read> primitives:array<Prim>;
@group(0) @binding(2) var<storage,read_write> samples:array<vec4<f32>>;
fn circle(x:f32,y:f32,p:Prim)->f32{
    let dx=x-p.pos_radius.x;
    let dy=y-p.pos_radius.y;
    return sqrt(dx*dx+dy*dy)-p.pos_radius.z;
}
fn signed_field(x:f32,y:f32)->f32 {
    var d=circle(x,y,primitives[0]);
    for(var i:u32=1u;i<conf.dims.z;i=i+1u){
        let p=primitives[i];
        let v=circle(x,y,p);
        let op=u32(p.pos_radius.w);
        if(op==1u){d=max(d,-v);}
        else if(op==2u){d=max(d,v);}
        else if(op==3u){
            let k=0.08;
            let h=max(k-abs(d-v),0.0)/k;
            d=min(d,v)-h*h*k*0.25;
        }else{d=min(d,v);}
    }
    return d;
}
@compute @workgroup_size(8,8,1)
fn main(@builtin(global_invocation_id) gid:vec3<u32>){
    if(gid.x>=conf.dims.x||gid.y>=conf.dims.y){return;}
    let s=f32(min(conf.dims.x,conf.dims.y));
    let x=(f32(gid.x)+0.5-f32(conf.dims.x)*0.5)/s;
    let y=(f32(gid.y)+0.5-f32(conf.dims.y)*0.5)/s;
    let eps=1.0/s;
    let d=signed_field(x,y);
    let dx=(signed_field(x+eps,y)-signed_field(x-eps,y))*0.5;
    let dy=(signed_field(x,y+eps)-signed_field(x,y-eps))*0.5;
    let mag=sqrt(dx*dx+dy*dy);
    var nx=0.0;var ny=0.0;
    if(mag>1e-8){nx=dx/mag;ny=dy/mag;}
    let coverage=clamp(0.5-d/(1.2/s),0.0,1.0);
    samples[gid.y*conf.dims.x+gid.x]=vec4<f32>(d,nx,ny,coverage);
}
