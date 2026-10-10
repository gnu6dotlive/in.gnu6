// CP02 GPU-00: actual Vulkan compute with mesh-derived convex support planes.
// A tetrahedron's MAX signed planes is an IMPLICIT FIELD, NOT exact mesh SDF.
// The host must independently certify topology and upload its four face planes.
struct Items { values: array<vec4<f32>>, }
@group(0) @binding(0) var<storage, read> points: Items;
@group(0) @binding(1) var<storage, read> planes: Items;
@group(0) @binding(2) var<storage, read_write> result: Items;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) invocation:vec3<u32>) {
    let i=invocation.x;
    if (i >= arrayLength(&points.values)) {return;}
    let p=points.values[i].xyz;
    let sphere=distance(p, vec3<f32>(0.15,0.15,0.15))-0.30;
    var convex=-1e30;
    for (var j:u32=0u;j<4u;j=j+1u) {
        let plane=planes.values[j];
        convex=max(convex,dot(plane.xyz,p)+plane.w);
    }
    result.values[i]=vec4<f32>(
        min(sphere,convex),              // SDF union MeshSolid
        max(sphere,convex),              // SDF intersection MeshSolid
        max(sphere,-convex),             // SDF minus MeshSolid
        max(convex,-sphere)              // MeshSolid minus SDF
    );
}
