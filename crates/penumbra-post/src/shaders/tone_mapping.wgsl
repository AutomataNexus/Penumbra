// Tone mapping fragment shader

@group(0) @binding(0) var input_texture: texture_2d<f32>;
@group(0) @binding(1) var input_sampler: sampler;

struct ToneMappingParams {
    exposure: f32,
    mode: u32, // 0=ACES, 1=Reinhard, 2=Uncharted2, 3=Linear
    _pad0: u32,
    _pad1: u32,
};

@group(0) @binding(2) var<uniform> params: ToneMappingParams;

fn aces_film(x: vec3<f32>) -> vec3<f32> {
    let a = 2.51;
    let b = 0.03;
    let c = 2.43;
    let d = 0.59;
    let e = 0.14;
    return saturate((x * (a * x + b)) / (x * (c * x + d) + e));
}

fn reinhard(x: vec3<f32>) -> vec3<f32> {
    return x / (x + vec3<f32>(1.0));
}

fn uncharted2_partial(x: vec3<f32>) -> vec3<f32> {
    let A = 0.15;
    let B = 0.50;
    let C = 0.10;
    let D = 0.20;
    let E = 0.02;
    let F = 0.30;
    return ((x * (A * x + C * B) + D * E) / (x * (A * x + B) + D * F)) - E / F;
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(input_texture, input_sampler, in.uv).rgb;
    let exposed = color * params.exposure;

    var mapped: vec3<f32>;
    switch params.mode {
        case 0u: { mapped = aces_film(exposed); }
        case 1u: { mapped = reinhard(exposed); }
        case 2u: {
            let W = vec3<f32>(11.2);
            let numerator = uncharted2_partial(exposed);
            let denominator = uncharted2_partial(W);
            mapped = numerator / denominator;
        }
        default: { mapped = exposed; }
    }

    // Gamma correction
    mapped = pow(mapped, vec3<f32>(1.0 / 2.2));
    return vec4<f32>(mapped, 1.0);
}
