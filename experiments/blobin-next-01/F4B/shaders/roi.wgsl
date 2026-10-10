// F4B GPU-only region compositor. It does not compute F4 effects.
// Source is a fully materialized F4 output, validated by the host-side oracle.
struct Region {
    dims_origin: vec4<u32>, // canvas W/H and region x/y
    dims_extent: vec4<u32>, // region W/H, reserved
}
@group(0) @binding(0) var<uniform> roi: Region;
@group(0) @binding(1) var<storage, read> source: array<u32>;
@group(0) @binding(2) var<storage, read_write> destination: array<u32>;
@compute @workgroup_size(8,8,1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    if(id.x >= roi.dims_extent.x || id.y >= roi.dims_extent.y) { return; }
    let x=roi.dims_origin.z+id.x;
    let y=roi.dims_origin.w+id.y;
    if(x>=roi.dims_origin.x || y>=roi.dims_origin.y) { return; }
    let index=y*roi.dims_origin.x+x;
    destination[index]=source[index];
}
