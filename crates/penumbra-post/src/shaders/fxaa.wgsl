// FXAA (Fast Approximate Anti-Aliasing) fragment shader

@group(0) @binding(0) var input_texture: texture_2d<f32>;
@group(0) @binding(1) var input_sampler: sampler;

struct FxaaParams {
    texel_size: vec2<f32>,
    edge_threshold: f32,
    edge_threshold_min: f32,
};

@group(0) @binding(2) var<uniform> params: FxaaParams;

fn luminance(color: vec3<f32>) -> f32 {
    return dot(color, vec3<f32>(0.299, 0.587, 0.114));
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let center = textureSample(input_texture, input_sampler, in.uv);
    let luma_center = luminance(center.rgb);

    let luma_n = luminance(textureSample(input_texture, input_sampler, in.uv + vec2<f32>(0.0, -params.texel_size.y)).rgb);
    let luma_s = luminance(textureSample(input_texture, input_sampler, in.uv + vec2<f32>(0.0, params.texel_size.y)).rgb);
    let luma_e = luminance(textureSample(input_texture, input_sampler, in.uv + vec2<f32>(params.texel_size.x, 0.0)).rgb);
    let luma_w = luminance(textureSample(input_texture, input_sampler, in.uv + vec2<f32>(-params.texel_size.x, 0.0)).rgb);

    let luma_min = min(luma_center, min(min(luma_n, luma_s), min(luma_e, luma_w)));
    let luma_max = max(luma_center, max(max(luma_n, luma_s), max(luma_e, luma_w)));
    let luma_range = luma_max - luma_min;

    if luma_range < max(params.edge_threshold_min, luma_max * params.edge_threshold) {
        return center;
    }

    // Simplified FXAA: blend with neighbors along detected edge
    let filter = (luma_n + luma_s + luma_e + luma_w) * 0.25;
    let blend = clamp(abs(filter - luma_center) / luma_range, 0.0, 1.0);
    let final_blend = smoothstep(0.0, 1.0, blend) * 0.5;

    let avg = (
        textureSample(input_texture, input_sampler, in.uv + vec2<f32>(0.0, -params.texel_size.y)).rgb +
        textureSample(input_texture, input_sampler, in.uv + vec2<f32>(0.0, params.texel_size.y)).rgb +
        textureSample(input_texture, input_sampler, in.uv + vec2<f32>(params.texel_size.x, 0.0)).rgb +
        textureSample(input_texture, input_sampler, in.uv + vec2<f32>(-params.texel_size.x, 0.0)).rgb
    ) * 0.25;

    return vec4<f32>(mix(center.rgb, avg, final_blend), center.a);
}
