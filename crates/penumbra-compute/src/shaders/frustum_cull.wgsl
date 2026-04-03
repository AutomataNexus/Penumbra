// GPU frustum culling compute shader

struct CullParams {
    view_projection: mat4x4<f32>,
    instance_count: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
};

struct InstanceBounds {
    center: vec3<f32>,
    radius: f32,
};

struct DrawIndirect {
    vertex_count: u32,
    instance_count: atomic<u32>,
    first_vertex: u32,
    first_instance: u32,
};

@group(0) @binding(0) var<uniform> params: CullParams;
@group(0) @binding(1) var<storage, read> bounds: array<InstanceBounds>;
@group(0) @binding(2) var<storage, read_write> visible: array<u32>;
@group(0) @binding(3) var<storage, read_write> draw: DrawIndirect;

fn is_sphere_in_frustum(center: vec3<f32>, radius: f32, vp: mat4x4<f32>) -> bool {
    let clip = vp * vec4<f32>(center, 1.0);
    // Check against all six frustum planes (simplified clip-space test)
    let w = clip.w;
    if clip.x < -w - radius || clip.x > w + radius { return false; }
    if clip.y < -w - radius || clip.y > w + radius { return false; }
    if clip.z < -radius || clip.z > w + radius { return false; }
    return true;
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let idx = gid.x;
    if idx >= params.instance_count {
        return;
    }
    let b = bounds[idx];
    if is_sphere_in_frustum(b.center, b.radius, params.view_projection) {
        let out_idx = atomicAdd(&draw.instance_count, 1u);
        visible[out_idx] = idx;
    }
}
