// F4 clean-room GPU postprocessing. Each dispatch executes one typed stage;
// F3 owns source material, F2 owns binary topology and scene identity.
struct Config { dimensions:vec4<u32>, control:vec4<u32> }
@group(0) @binding(0) var<uniform> config:Config;
@group(0) @binding(1) var<storage,read> source:array<u32>;
@group(0) @binding(2) var<storage,read> current:array<u32>;
@group(0) @binding(3) var<storage,read_write> output:array<u32>;
@group(0) @binding(4) var<storage,read> mask:array<u32>;
@group(0) @binding(5) var<storage,read> history:array<u32>;
fn channel(pixel:u32,shift:u32)->u32 {return (pixel>>shift)&255u;}
fn luma(x:u32,y:u32)->f32 {
    let p=source[y*config.dimensions.x+x];
    return f32(channel(p,0u)+channel(p,8u)+channel(p,16u))/765.0;
}
fn row_mean(x:u32,y:u32,r:i32,bright:bool)->f32{
    var sum=0.0;
    var n=0.0;
    for(var dx:i32=-8;dx<=8;dx=dx+1){
        if(abs(dx)>r) {continue;}
        let sx=i32(x)+dx;
        if(sx<0||sx>=i32(config.dimensions.x)){continue;}
        let v=luma(u32(sx),y);
        sum=sum+select(v,max(v-0.45,0.0),bright);
        n=n+1.0;
    }
    return sum/n;
}
fn blur(x:u32,y:u32,r:i32,bright:bool)->f32{
    if(r==0){
        let v=luma(x,y);
        return select(v,max(v-0.45,0.0),bright);
    }
    var sum=0.0;
    var n=0.0;
    for(var dy:i32=-8;dy<=8;dy=dy+1){
        if(abs(dy)>r){continue;}
        let sy=i32(y)+dy;
        if(sy<0||sy>=i32(config.dimensions.y)){continue;}
        sum=sum+row_mean(x,u32(sy),r,bright);
        n=n+1.0;
    }
    return sum/n;
}
fn satadd(v:u32,inc:u32)->u32 {return min(255u,v+inc);}
fn satsub(v:u32,dec:u32)->u32 {return select(0u,v-dec,v>=dec);}
@compute @workgroup_size(8,8,1)
fn main(@builtin(global_invocation_id) gid:vec3<u32>) {
    if(gid.x>=config.dimensions.x||gid.y>=config.dimensions.y) {return;}
    let x=gid.x;let y=gid.y;let index=y*config.dimensions.x+x;
    let old=current[index];var next_pixel=old;
    let stage=config.control.x;
    let radius=i32(config.control.y);
    if(stage==0u){
        if(x>=2u&&y>=2u&&mask[(y-2u)*config.dimensions.x+(x-2u)]!=0u&&mask[index]==0u){
            let r=satsub(channel(old,0u),12u);
            let g=satsub(channel(old,8u),12u);
            let b=satsub(channel(old,16u),12u);
            next_pixel=r|(g<<8u)|(b<<16u)|(255u<<24u);
        }
    } else if(stage==1u||stage==2u){
        var inc=0u;
        if(stage==1u&&mask[index]!=0u){inc=u32(round(blur(x,y,radius,false)*10.0));}
        if(stage==2u){inc=u32(round(blur(x,y,radius,true)*18.0));}
        let r=satadd(channel(old,0u),inc);
        let g=satadd(channel(old,8u),inc);
        let b=satadd(channel(old,16u),inc);
        next_pixel=r|(g<<8u)|(b<<16u)|(255u<<24u);
    } else if(stage==3u&&config.control.z!=0u&&config.control.w==0u){
        let previous=history[index];
        let r=(channel(old,0u)*3u+channel(previous,0u))/4u;
        let g=(channel(old,8u)*3u+channel(previous,8u))/4u;
        let b=(channel(old,16u)*3u+channel(previous,16u))/4u;
        next_pixel=r|(g<<8u)|(b<<16u)|(255u<<24u);
    }
    output[index]=next_pixel;
}
