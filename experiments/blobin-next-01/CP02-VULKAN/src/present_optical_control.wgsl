// ORACLE-06: 2D semantic control marker on top of independent Vulkan 3D view.
// Derived from the qualified read-only scene. NOT the 3D cavity's outline.
// The original 350ms upper-left optical ROI is independent and unchanged.
@group(0) @binding(0) var image: texture_2d<f32>;
@group(0) @binding(1) var image_sampler: sampler;
@group(0) @binding(2) var<uniform> optical_state: vec4<f32>;
@group(0) @binding(3) var<uniform> control_rect_uv: vec4<f32>;
struct Out { @builtin(position) pos:vec4<f32>, @location(0) uv:vec2<f32>, }
@vertex fn vs(@builtin(vertex_index) index:u32)->Out {
 let p=array<vec2<f32>,3>(vec2<f32>(-1.0,-1.0),vec2<f32>(3.0,-1.0),vec2<f32>(-1.0,3.0));
 var result:Out;
 result.pos=vec4<f32>(p[index],0.0,1.0);
 result.uv=vec2<f32>((p[index].x+1.0)*0.5,1.0-(p[index].y+1.0)*0.5);
 return result;
}
@fragment fn fs(in:Out)->@location(0) vec4<f32>{
 if (in.uv.x>=0.10 && in.uv.x<=0.26 && in.uv.y>=0.10 && in.uv.y<=0.26) {
    let luminance=select(0.0,1.0,optical_state.x>0.5);
    return vec4<f32>(luminance,luminance,luminance,1.0);
 }
 if (in.uv.x>=control_rect_uv.x && in.uv.x<=control_rect_uv.z &&
     in.uv.y>=control_rect_uv.y && in.uv.y<=control_rect_uv.w) {
    let edge=(in.uv.x-control_rect_uv.x)<0.002 ||
             (control_rect_uv.z-in.uv.x)<0.002 ||
             (in.uv.y-control_rect_uv.y)<0.003 ||
             (control_rect_uv.w-in.uv.y)<0.003;
    return select(vec4<f32>(0.10,0.18,0.26,1.0),
                  vec4<f32>(0.16,0.99,0.78,1.0),edge);
 }
 return textureSample(image,image_sampler,in.uv);
}
