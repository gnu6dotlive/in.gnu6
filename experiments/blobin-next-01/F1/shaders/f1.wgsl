// Original F1 reference field, mirrored by Rust src/lib.rs.
// Output is RGBA8Unorm-compatible packed bytes (little endian).
struct Frame {
    dims: vec4<u32>,  // pixel_width, pixel_height, logical_width, logical_height
    params: vec4<f32>, // dpr, phase, focus(0|1), reserved
}
@group(0) @binding(0) var<uniform> frame: Frame;
@group(0) @binding(1) var<storage, read_write> pixels: array<u32>;

fn encode_channel(v: f32) -> u32 {
    return u32(round(clamp(v, 0.0, 1.0) * 255.0));
}
@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= frame.dims.x || gid.y >= frame.dims.y) { return; }
    let lw = f32(frame.dims.z);
    let lh = f32(frame.dims.w);
    let dpr = frame.params.x;
    let scale = min(lw, lh);
    let qx = ((f32(gid.x) + 0.5) / dpr - lw * 0.5) / scale;
    let qy = ((f32(gid.y) + 0.5) / dpr - lh * 0.5) / scale;
    let theta = atan2(qy, qx);
    let radius = 0.36 + 0.023 * sin(theta * 5.0 + frame.params.y)
        + 0.012 * cos(theta * 3.0 - frame.params.y);
    let outer = sqrt(qx*qx + qy*qy) - radius;
    let hx = qx - 0.12;
    let hy = qy + 0.03;
    let hole = sqrt(hx*hx + hy*hy) - 0.095;
    let d = max(outer, -hole);
    let aa = 1.2 / (scale * dpr);
    let coverage = clamp(0.5 - d / aa, 0.0, 1.0);
    let bevel = clamp(1.0 - abs(d) / 0.035, 0.0, 1.0);
    let len = max(sqrt(qx*qx + qy*qy), 0.00001);
    let incidence = clamp(0.52 - (qx/len*0.33 - qy/len*0.52), 0.0, 1.0);
    let focus_power = select(0.12, 0.22, frame.params.z > 0.5);
    let illumination = bevel * incidence * focus_power;
    let fill = clamp(vec3<f32>(0.11, 0.28, 0.31) + vec3<f32>(illumination), vec3<f32>(0.0), vec3<f32>(1.0));
    let back = vec3<f32>(0.025, 0.033, 0.043);
    let rgb = back*(1.0-coverage) + fill*coverage;
    let r = encode_channel(rgb.r);
    let g = encode_channel(rgb.g);
    let b = encode_channel(rgb.b);
    pixels[gid.y*frame.dims.x + gid.x] = r | (g<<8u) | (b<<16u) | (255u<<24u);
}
