// Penumbra Globe Demo — PBR sphere with procedural Earth coloring + atmosphere

struct Uniforms {
    view_proj: mat4x4<f32>,
    model: mat4x4<f32>,
    camera_pos: vec4<f32>,
    light_dir: vec4<f32>,
    params: vec4<f32>,  // x=time, y=metallic, z=roughness, w=is_atmosphere
};

@group(0) @binding(0) var<uniform> u: Uniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) color: vec3<f32>,
    @location(3) local_pos: vec3<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    let world_pos = (u.model * vec4(in.position, 1.0)).xyz;
    let world_normal = normalize((u.model * vec4(in.normal, 0.0)).xyz);

    var out: VertexOutput;
    out.clip_position = u.view_proj * vec4(world_pos, 1.0);
    out.world_pos = world_pos;
    out.world_normal = world_normal;
    out.color = in.color;
    out.local_pos = in.position;
    return out;
}

const PI: f32 = 3.14159265359;

// Simple hash for procedural noise
fn hash(p: vec2<f32>) -> f32 {
    let h = dot(p, vec2(127.1, 311.7));
    return fract(sin(h) * 43758.5453123);
}

// Value noise
fn noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    return mix(
        mix(hash(i), hash(i + vec2(1.0, 0.0)), u.x),
        mix(hash(i + vec2(0.0, 1.0)), hash(i + vec2(1.0, 1.0)), u.x),
        u.y
    );
}

// Fractal brownian motion
fn fbm(p: vec2<f32>) -> f32 {
    var val = 0.0;
    var amp = 0.5;
    var pos = p;
    for (var i = 0; i < 5; i++) {
        val += amp * noise(pos);
        pos *= 2.0;
        amp *= 0.5;
    }
    return val;
}

// Procedural Earth color from lat/lon
fn earth_color(pos: vec3<f32>) -> vec3<f32> {
    let n = normalize(pos);
    let lat = asin(n.y);
    let lon = atan2(n.z, n.x);

    // Continental noise
    let continental = fbm(vec2(lon * 3.0, lat * 3.0));
    let detail = fbm(vec2(lon * 12.0, lat * 12.0));

    // Water vs land threshold
    let land_mask = smoothstep(0.38, 0.42, continental + detail * 0.15);

    // Water colors (deep vs shallow)
    let deep_water = vec3(0.02, 0.06, 0.18);
    let shallow_water = vec3(0.05, 0.15, 0.35);
    let water = mix(deep_water, shallow_water, smoothstep(0.25, 0.38, continental));

    // Land colors by latitude
    let abs_lat = abs(lat);

    // Ice caps
    let ice = vec3(0.9, 0.92, 0.95);
    // Tundra
    let tundra = vec3(0.45, 0.48, 0.38);
    // Forest
    let forest = vec3(0.08, 0.22, 0.06);
    // Grassland
    let grass = vec3(0.25, 0.38, 0.12);
    // Desert
    let desert = vec3(0.65, 0.52, 0.32);
    // Tropical
    let tropical = vec3(0.05, 0.28, 0.08);

    var land: vec3<f32>;
    if abs_lat > 1.2 {
        land = ice;
    } else if abs_lat > 1.0 {
        land = mix(tundra, ice, smoothstep(1.0, 1.2, abs_lat));
    } else if abs_lat > 0.7 {
        land = mix(forest, tundra, smoothstep(0.7, 1.0, abs_lat));
    } else if abs_lat > 0.4 {
        let moisture = fbm(vec2(lon * 5.0 + 10.0, lat * 5.0));
        land = mix(desert, grass, moisture);
    } else {
        let moisture = fbm(vec2(lon * 6.0 + 5.0, lat * 6.0));
        land = mix(desert, tropical, smoothstep(0.3, 0.6, moisture));
    }

    // Add terrain detail variation
    land = land * (0.85 + detail * 0.3);

    return mix(water, land, land_mask);
}

// PBR lighting (simplified)
fn pbr_light(n: vec3<f32>, v: vec3<f32>, l: vec3<f32>, albedo: vec3<f32>, metallic: f32, roughness: f32) -> vec3<f32> {
    let h = normalize(v + l);
    let n_dot_l = max(dot(n, l), 0.0);
    let n_dot_h = max(dot(n, h), 0.0);
    let n_dot_v = max(dot(n, v), 0.001);

    // Specular: simplified GGX
    let a = roughness * roughness;
    let a2 = a * a;
    let denom = n_dot_h * n_dot_h * (a2 - 1.0) + 1.0;
    let D = a2 / (PI * denom * denom);

    // Fresnel
    let f0 = mix(vec3(0.04), albedo, metallic);
    let F = f0 + (1.0 - f0) * pow(1.0 - max(dot(v, h), 0.0), 5.0);

    let specular = D * F * 0.25 / (n_dot_v + 0.0001);
    let diffuse = (1.0 - F) * (1.0 - metallic) * albedo / PI;

    return (diffuse + specular) * n_dot_l * vec3(1.0, 0.97, 0.92) * 1.2;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let n = normalize(in.world_normal);
    let v = normalize(u.camera_pos.xyz - in.world_pos);
    let l = normalize(-u.light_dir.xyz);

    let is_atmo = u.params.w;

    if is_atmo > 0.5 {
        // Atmosphere shell: rim glow
        let rim = 1.0 - max(dot(n, v), 0.0);
        let glow = pow(rim, 3.0) * 0.8;
        let scatter = pow(rim, 1.5) * 0.3;

        // Blue atmosphere with sunset tint near terminator
        let sun_facing = max(dot(n, l), 0.0);
        let atmo_color = mix(
            vec3(0.15, 0.3, 0.8),   // blue sky
            vec3(0.9, 0.4, 0.15),   // sunset orange
            pow(1.0 - sun_facing, 4.0) * rim
        );

        let alpha = glow + scatter;
        return vec4(atmo_color * (glow + scatter * sun_facing), alpha * 0.6);
    }

    // Globe surface
    var albedo = in.color;

    // If color is white-ish, use procedural earth
    if in.color.r > 0.9 && in.color.g > 0.9 {
        albedo = earth_color(in.local_pos);
    }

    let metallic = u.params.y;
    let roughness = u.params.z;

    // Ambient (sky light)
    let sky_amount = max(dot(n, vec3(0.0, 1.0, 0.0)), 0.0) * 0.5 + 0.5;
    var color = albedo * mix(vec3(0.01, 0.01, 0.02), vec3(0.04, 0.06, 0.12), sky_amount);

    // Sun light
    color += pbr_light(n, v, l, albedo, metallic, roughness);

    // Night side city lights (faint orange dots)
    let night = max(-dot(n, l), 0.0);
    if night > 0.1 {
        let city_noise = fbm(vec2(in.local_pos.x * 30.0, in.local_pos.z * 30.0));
        let land = earth_color(in.local_pos);
        let is_land = step(0.15, land.g); // crude land detection
        let city = smoothstep(0.6, 0.65, city_noise) * is_land * night;
        color += vec3(1.0, 0.7, 0.3) * city * 0.4;
    }

    // ACES tone mapping
    color = clamp((color * (2.51 * color + 0.03)) / (color * (2.43 * color + 0.59) + 0.14), vec3(0.0), vec3(1.0));
    color = pow(color, vec3(1.0 / 2.2));

    return vec4(color, 1.0);
}
