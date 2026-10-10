// Opt-in local witness target. The scene image is unchanged outside ROI.
// A 350ms pulse comes only from a real compositor-delivered pointer press;
// this is NOT a semantic scene mutation or a physical-photon measurement.
@group(0) @binding(0) var image: texture_2d<f32>;
@group(0) @binding(1) var image_sampler: sampler;
@group(0) @binding(2) var<uniform> optical_state: vec4<f32>;
struct Out { @builtin(position) pos:vec4<f32>, @location(0) uv:vec2<f32>, }
@vertex fn vs(@builtin(vertex_index) index:u32)->Out {
 let p=array<vec2<f32>,3>(vec2<f32>(-1.0,-1.0),vec2<f32>(3.0,-1.0),vec2<f32>(-1.0,3.0));
 var result:Out;
 result.pos=vec4<f32>(p[index],0.0,1.0);
 result.uv=vec2<f32>((p[index].x+1.0)*0.5,1.0-(p[index].y+1.0)*0.5);
 return result;
}
@fragment fn fs(in:Out)->@location(0) vec4<f32>{
 // Fixed 16% patch upper-left: dark when off, white when on.
 if (in.uv.x>=0.10 && in.uv.x<=0.26 && in.uv.y>=0.10 && in.uv.y<=0.26) {
    let luminance=select(0.0,1.0,optical_state.x>0.5);
    return vec4<f32>(luminance,luminance,luminance,1.0);
 }
 return textureSample(image,image_sampler,in.uv);
}
