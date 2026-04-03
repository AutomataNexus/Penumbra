// SDF text rendering shader

struct CameraUniforms {
    view_projection: mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> camera: CameraUniforms;
@group(0) @binding(1) var font_texture: texture_2d<f32>;
@group(0) @binding(2) var font_sampler: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.position = camera.view_projection * vec4<f32>(input.position, 1.0);
    out.uv = input.uv;
    out.color = input.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let distance = textureSample(font_texture, font_sampler, in.uv).r;

    // SDF rendering with smoothstep for anti-aliasing
    let edge = 0.5;
    let smoothing = fwidth(distance) * 0.5;
    let alpha = smoothstep(edge - smoothing, edge + smoothing, distance);

    return vec4<f32>(in.color.rgb, in.color.a * alpha);
}
