// Shadow mapping shader

struct ShadowUniforms {
    light_space_matrix: mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> shadow: ShadowUniforms;

struct ModelUniforms {
    model: mat4x4<f32>,
};

@group(1) @binding(0) var<uniform> model: ModelUniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
};

@vertex
fn vs_shadow(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.position = shadow.light_space_matrix * model.model * vec4<f32>(input.position, 1.0);
    return out;
}

// PCF shadow sampling
fn sample_shadow_pcf(
    shadow_map: texture_depth_2d,
    shadow_sampler: sampler_comparison,
    light_space_pos: vec4<f32>,
    bias: f32,
    texel_size: f32,
) -> f32 {
    let proj = light_space_pos.xyz / light_space_pos.w;
    let uv = proj.xy * 0.5 + 0.5;
    let depth = proj.z - bias;

    if uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0 || depth > 1.0 {
        return 1.0;
    }

    var shadow_val = 0.0;
    for (var y = -1; y <= 1; y = y + 1) {
        for (var x = -1; x <= 1; x = x + 1) {
            let offset = vec2<f32>(f32(x), f32(y)) * texel_size;
            shadow_val += textureSampleCompare(shadow_map, shadow_sampler, uv + offset, depth);
        }
    }
    return shadow_val / 9.0;
}
