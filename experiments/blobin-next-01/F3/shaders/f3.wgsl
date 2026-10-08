// BLOBIN F3 original clean-room WGSL, no historical shader imports.
// Both renderers evaluate F2 geometry independently. GPU never owns scene identity.
struct Frame {
    dims: vec4<u32>, // physical W/H, logical W/H
    params: vec4<f32>, // DPR, phase, topology layout, material ID
    extra: vec4<f32>, // synthetic heat strength, unused
}
@group(0) @binding(0) var<uniform> frame: Frame;
@group(0) @binding(1) var<storage, read_write> pixels: array<u32>;
@group(0) @binding(2) var<storage, read_write> analytics: array<vec4<f32>>;

fn coords(fx:f32, fy:f32)->vec2<f32> {
    let scale=f32(min(frame.dims.z,frame.dims.w));
    return (vec2<f32>(fx,fy)/frame.params.x
      -vec2<f32>(f32(frame.dims.z)*0.5,f32(frame.dims.w)*0.5))/scale;
}
fn lobe(x:f32,y:f32,cx:f32,phase:f32)->f32 {
    let xx=x-cx;
    let theta=atan2(y,xx);
    let r=0.235+0.008*sin(3.0*theta+phase)+0.006*cos(5.0*theta-phase);
    return sqrt(xx*xx+y*y)-r;
}
fn field(fx:f32,fy:f32)->f32 {
    let q=coords(fx,fy);
    let sep=select(0.15,0.31,frame.params.z<0.5);
    let combined=min(lobe(q.x,q.y,-sep,frame.params.y),
                     lobe(q.x,q.y,sep,frame.params.y));
    return select(combined,max(combined,0.055-length(q)),frame.params.z>1.5);
}
fn height_at(fx:f32,fy:f32)->f32 {
    let d=field(fx,fy);
    let aa=1.2/(f32(min(frame.dims.z,frame.dims.w))*frame.params.x);
    let coverage=clamp(0.5-d/aa,0.0,1.0);
    let rim=clamp(1.0-abs(d)/0.045,0.0,1.0);
    return clamp(0.11*coverage+0.24*rim*coverage,0.0,0.35);
}
fn heat_at(fx:f32,fy:f32)->f32 {
    let q=coords(fx,fy);
    let a=exp(-((q.x+0.16)*(q.x+0.16)+(q.y+0.09)*(q.y+0.09))*46.0);
    let b=exp(-((q.x-0.19)*(q.x-0.19)+(q.y-0.07)*(q.y-0.07))*68.0);
    return clamp(frame.extra.x*(0.78*a+0.48*b),0.0,1.0);
}
fn srgb(v:f32)->f32 {
    let q=clamp(v,0.0,1.0);
    return select(1.055*pow(q,1.0/2.4)-0.055,12.92*q,q<=0.0031308);
}
fn over(fg:vec4<f32>,bg:vec4<f32>)->vec4<f32> {
    let fa=clamp(fg.a,0.0,1.0);
    let ba=clamp(bg.a,0.0,1.0);
    return vec4<f32>(fg.rgb*fa+bg.rgb*ba*(1.0-fa),fa+ba*(1.0-fa));
}
fn channel(v:f32)->u32 {return u32(round(clamp(v,0.0,1.0)*255.0));}
@compute @workgroup_size(8,8,1)
fn main(@builtin(global_invocation_id) gid:vec3<u32>) {
    if(gid.x>=frame.dims.x||gid.y>=frame.dims.y) {return;}
    let fx=f32(gid.x)+0.5;
    let fy=f32(gid.y)+0.5;
    let d=field(fx,fy);
    let inside=select(0.0,1.0,d<0.0);
    let aa=1.2/(f32(min(frame.dims.z,frame.dims.w))*frame.params.x);
    let coverage=clamp(0.5-d/aa,0.0,1.0);
    let h=height_at(fx,fy);
    let dx=(height_at(fx+1.0,fy)-height_at(fx-1.0,fy))*0.5;
    let dy=(height_at(fx,fy+1.0)-height_at(fx,fy-1.0))*0.5;
    let normal=normalize(vec3<f32>(-dx*2.0,-dy*2.0,0.20));
    let light=normalize(vec3<f32>(-0.42,-0.58,0.92));
    let ndotl=max(dot(normal,light),0.0);
    let rim=clamp(1.0-abs(d)/0.045,0.0,1.0);
    let q=coords(fx,fy);
    let material_id=u32(frame.params.w);
    var base=vec3<f32>(0.07,0.18,0.26);
    var spec=0.34;
    if(material_id==1u) {base=vec3<f32>(0.20,0.10,0.29); spec=0.15;}
    if(material_id==2u) {base=vec3<f32>(0.23,0.26,0.24); spec=0.27;}
    if(material_id==3u) {base=vec3<f32>(0.34,0.29,0.20); spec=0.05;}
    var grain=0.0;
    if(material_id==2u) {grain=0.016*sin(q.y*172.0+frame.params.y);}
    if(material_id==3u) {grain=0.013*sin(q.x*149.0+q.y*73.0);}
    let relief=rim*ndotl*spec;
    let gradient=0.052*clamp(0.5-q.y,-0.5,1.5);
    let material=vec4<f32>(
        clamp(base+vec3<f32>(relief+gradient+grain),vec3<f32>(0.0),vec3<f32>(1.0)),coverage);
    let heat=heat_at(fx,fy);
    let layer=over(material,vec4<f32>(0.018,0.026,0.035,1.0));
    let result=over(vec4<f32>(0.62,0.20,0.07,heat*coverage*0.24),layer);
    let rgb=vec3<f32>(srgb(result.r),srgb(result.g),srgb(result.b));
    let i=gid.y*frame.dims.x+gid.x;
    pixels[i]=channel(rgb.r)|(channel(rgb.g)<<8u)|(channel(rgb.b)<<16u)|(255u<<24u);
    analytics[i]=vec4<f32>(h,heat,inside,normal.z);
}
