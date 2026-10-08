// Newly authored F2 WGSL implementation; CPU Rust remains a separate falsifiable oracle.
struct Frame {
    dims: vec4<u32>, // physical width/height, logical width/height
    params: vec4<f32>, // DPR, phase, layout_code, reserved
}
@group(0) @binding(0) var<uniform> frame: Frame;
@group(0) @binding(1) var<storage, read_write> pixels: array<u32>;

fn lobe(x: f32, y: f32, cx: f32, phase: f32) -> f32 {
    let px = x - cx;
    let theta = atan2(y, px);
    let r = 0.235 + 0.008*sin(3.0*theta + phase) + 0.006*cos(5.0*theta - phase);
    return sqrt(px*px + y*y) - r;
}
fn channel(v: f32) -> u32 { return u32(round(clamp(v, 0.0, 1.0) * 255.0)); }
@compute @workgroup_size(8,8,1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= frame.dims.x || gid.y >= frame.dims.y) { return; }
    let lw = f32(frame.dims.z);
    let lh = f32(frame.dims.w);
    let dpr = frame.params.x;
    let phase = frame.params.y;
    let mode = frame.params.z;
    let scale = min(lw,lh);
    let x = ((f32(gid.x)+0.5)/dpr - lw*0.5)/scale;
    let y = ((f32(gid.y)+0.5)/dpr - lh*0.5)/scale;
    let separation = select(0.15,0.31,mode<0.5);
    let da = lobe(x,y,-separation,phase);
    let db = lobe(x,y,separation,phase);
    let merged_field = min(da,db);
    let d = select(merged_field,max(merged_field,0.055 - sqrt(x*x+y*y)),mode>1.5);
    let aa = 1.2 / (scale*dpr);
    let coverage = clamp(0.5-d/aa,0.0,1.0);
    let bevel = clamp(1.0-abs(d)/0.028,0.0,1.0);
    let len = max(sqrt(x*x+y*y),0.00001);
    let incidence = clamp(0.52-(x/len*0.33-y/len*0.52),0.0,1.0);
    let illumination = bevel*incidence*0.22;
    let fill = clamp(vec3<f32>(0.11,0.28,0.31)+vec3<f32>(illumination),vec3<f32>(0.0),vec3<f32>(1.0));
    let back = vec3<f32>(0.025,0.033,0.043);
    let rgb = back*(1.0-coverage)+fill*coverage;
    pixels[gid.y*frame.dims.x+gid.x] =
        channel(rgb.r) | (channel(rgb.g)<<8u) | (channel(rgb.b)<<16u) | (255u<<24u);
}
