// F4C original compute: every pass operates exclusively on a bounded ROI.
// Intermediate stage outputs are persistent GPU buffers. F3 still supplies full RGBA input.
struct Config {
    dimensions: vec4<u32>, // width height roi x roi y
    control: vec4<u32>, // stage, radius, hasHistory, reducedMotion
    extent: vec4<u32>, // ROI width/height and reserved
}
@group(0) @binding(0) var<uniform> conf: Config;
@group(0) @binding(1) var<storage, read> original: array<u32>;
@group(0) @binding(2) var<storage, read> previous_stage: array<u32>;
@group(0) @binding(3) var<storage, read_write> stage_result: array<u32>;
@group(0) @binding(4) var<storage, read> mask: array<u32>;
@group(0) @binding(5) var<storage, read> history: array<u32>;

fn channel(pixel:u32,shift:u32)->u32 { return (pixel>>shift)&255u; }
fn luma(x:u32,y:u32)->f32 {
    let p=original[y*conf.dimensions.x+x];
    return f32(channel(p,0u)+channel(p,8u)+channel(p,16u))/765.0;
}
fn rowmean(x:u32,y:u32,r:i32,bright:bool)->f32 {
    var sum=0.0;
    var count=0.0;
    for(var dx:i32=-8;dx<=8;dx=dx+1){
        if(abs(dx)>r){continue;}
        let xx=i32(x)+dx;
        if(xx<0 || xx>=i32(conf.dimensions.x)){continue;}
        let val=luma(u32(xx),y);
        sum+=select(val,max(val-0.45,0.0),bright);
        count+=1.0;
    }
    return sum/count;
}
fn blur(x:u32,y:u32,r:i32,bright:bool)->f32 {
    if(r==0){
        let v=luma(x,y);
        return select(v,max(v-0.45,0.0),bright);
    }
    var sum=0.0;
    var count=0.0;
    for(var dy:i32=-8;dy<=8;dy=dy+1){
        if(abs(dy)>r){continue;}
        let yy=i32(y)+dy;
        if(yy<0||yy>=i32(conf.dimensions.y)){continue;}
        sum+=rowmean(x,u32(yy),r,bright);
        count+=1.0;
    }
    return sum/count;
}
fn satadd(v:u32,a:u32)->u32{return min(255u,v+a);}
fn satsub(v:u32,a:u32)->u32{return select(0u,v-a,v>=a);}
@compute @workgroup_size(8,8,1)
fn main(@builtin(global_invocation_id) id:vec3<u32>) {
    if(id.x>=conf.extent.x || id.y>=conf.extent.y){return;}
    let x=conf.dimensions.z+id.x;
    let y=conf.dimensions.w+id.y;
    if(x>=conf.dimensions.x || y>=conf.dimensions.y){return;}
    let index=y*conf.dimensions.x+x;
    let old=previous_stage[index];
    var next_pixel=old;
    let stage=conf.control.x;
    let radius=i32(conf.control.y);
    if(stage==0u){
        if(x>=2u && y>=2u && mask[(y-2u)*conf.dimensions.x+x-2u]!=0u && mask[index]==0u){
            let r=satsub(channel(old,0u),12u);
            let g=satsub(channel(old,8u),12u);
            let b=satsub(channel(old,16u),12u);
            next_pixel=r|(g<<8u)|(b<<16u)|(255u<<24u);
        }
    } else if(stage==1u || stage==2u){
        var inc=0u;
        if(stage==1u && mask[index]!=0u){inc=u32(round(blur(x,y,radius,false)*10.0));}
        if(stage==2u){inc=u32(round(blur(x,y,radius,true)*18.0));}
        let r=satadd(channel(old,0u),inc);
        let g=satadd(channel(old,8u),inc);
        let b=satadd(channel(old,16u),inc);
        next_pixel=r|(g<<8u)|(b<<16u)|(255u<<24u);
    } else if(stage==3u && conf.control.z!=0u && conf.control.w==0u){
        let p=history[index];
        let r=(channel(old,0u)*3u+channel(p,0u))/4u;
        let g=(channel(old,8u)*3u+channel(p,8u))/4u;
        let b=(channel(old,16u)*3u+channel(p,16u))/4u;
        next_pixel=r|(g<<8u)|(b<<16u)|(255u<<24u);
    }
    stage_result[index]=next_pixel;
}
