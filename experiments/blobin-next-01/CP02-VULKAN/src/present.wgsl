@group(0) @binding(0) var image:texture_2d<f32>;
@group(0) @binding(1) var image_sampler:sampler;
struct Out { @builtin(position) pos:vec4<f32>, @location(0) uv:vec2<f32>, }
@vertex fn vs(@builtin(vertex_index) index:u32)->Out {
    let p=array<vec2<f32>,3>(vec2<f32>(-1.0,-1.0),vec2<f32>(3.0,-1.0),vec2<f32>(-1.0,3.0));
    var result:Out;
    result.pos=vec4<f32>(p[index],0.0,1.0);
    result.uv=vec2<f32>((p[index].x+1.0)*0.5,1.0-(p[index].y+1.0)*0.5);
    return result;
}
@fragment fn fs(in:Out)->@location(0) vec4<f32>{
    return textureSample(image,image_sampler,in.uv);
}
