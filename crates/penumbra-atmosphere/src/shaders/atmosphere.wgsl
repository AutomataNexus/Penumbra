// Atmospheric scattering sky shader

const PI: f32 = 3.14159265359;

struct AtmosphereUniforms {
    sun_direction: vec4<f32>,
    rayleigh_scattering: vec4<f32>,  // xyz = coefficients, w = scale height
    mie_params: vec4<f32>,           // x = scattering, y = absorption, z = asymmetry (g), w = scale height
    planet_params: vec4<f32>,        // x = planet radius, y = atmo height, z = sun intensity
};

@group(0) @binding(0) var<uniform> atmo: AtmosphereUniforms;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) view_dir: vec3<f32>,
};

// Rayleigh phase function
fn rayleigh_phase(cos_theta: f32) -> f32 {
    return (3.0 / (16.0 * PI)) * (1.0 + cos_theta * cos_theta);
}

// Henyey-Greenstein phase function for Mie scattering
fn mie_phase(cos_theta: f32, g: f32) -> f32 {
    let g2 = g * g;
    let num = 3.0 * (1.0 - g2) * (1.0 + cos_theta * cos_theta);
    let denom = (8.0 * PI) * (2.0 + g2) * pow(1.0 + g2 - 2.0 * g * cos_theta, 1.5);
    return num / denom;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let view_dir = normalize(in.view_dir);
    let sun_dir = normalize(atmo.sun_direction.xyz);
    let cos_theta = dot(view_dir, sun_dir);

    let rayleigh = atmo.rayleigh_scattering.xyz;
    let mie_scat = atmo.mie_params.x;
    let g = atmo.mie_params.z;
    let sun_intensity = atmo.planet_params.z;

    let r_phase = rayleigh_phase(cos_theta);
    let m_phase = mie_phase(cos_theta, g);

    // Simplified single-scattering approximation
    let optical_depth = 1.0; // Placeholder
    let rayleigh_color = rayleigh * r_phase * optical_depth;
    let mie_color = vec3<f32>(mie_scat * m_phase * optical_depth);

    let sky = (rayleigh_color + mie_color) * sun_intensity;

    // Tone-map to [0,1]
    let mapped = vec3<f32>(1.0) - exp(-sky);
    return vec4<f32>(mapped, 1.0);
}
