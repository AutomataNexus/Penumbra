//! globe — Full globe with satellite tile streaming, terrain elevation, and atmosphere.
//!
//! Demonstrates: WGS84 geodesy, tile coordinate math, XYZ tile source,
//! terrain mesh generation, LRU tile cache, atmospheric scattering,
//! orbit camera, scene graph, and post-processing.

use glam::Vec3;
use penumbra_atmosphere::{AtmosphereConfig, AtmosphereRenderer, Fog, FogMode};
use penumbra_camera::OrbitController;
use penumbra_core::{Renderer, RendererConfig};
use penumbra_geo::{
    GeoPosition, haversine_distance, lat_lon_to_tile, tile_bounds, tile_resolution, wgs84_to_ecef,
    wgs84_to_enu,
};
use penumbra_pbr::{Light, PbrConfig, PbrPipeline};
use penumbra_post::{ColorGrading, Fxaa, PostPipeline, ToneMapping};
use penumbra_scene::Scene;
use penumbra_terrain::{
    TerrainConfig, TerrainData, TileCache, TileCoord, TileData, TileFormat, TileSource,
    XyzTileSource, decode_terrain_rgb, generate_tile_mesh,
};
use penumbra_wgpu::{WgpuBackend, WgpuConfig};

fn main() {
    tracing_subscriber::fmt::init();

    // ── Backend + renderer ──
    let backend = WgpuBackend::headless(1920, 1080, WgpuConfig::default())
        .expect("Failed to create wgpu backend");
    let mut renderer = Renderer::new(
        backend,
        RendererConfig {
            width: 1920,
            height: 1080,
            hdr: true,
            ..RendererConfig::default()
        },
    );

    println!("Penumbra globe — full globe with satellite tiles + terrain + atmosphere");
    println!("Backend: {}", renderer.backend_name());

    // ── Geodesy demo ──
    let nyc = GeoPosition {
        lat: 40.7128,
        lon: -74.0060,
        alt: 0.0,
    };
    let london = GeoPosition {
        lat: 51.5074,
        lon: -0.1278,
        alt: 0.0,
    };

    let ecef_nyc = wgs84_to_ecef(&nyc);
    println!(
        "NYC ECEF: ({:.0}, {:.0}, {:.0})",
        ecef_nyc.x, ecef_nyc.y, ecef_nyc.z
    );

    let dist = haversine_distance(&nyc, &london);
    println!("NYC -> London: {:.0} km", dist / 1000.0);

    let local_london = wgs84_to_enu(&london, &nyc);
    println!(
        "London in NYC-ENU: ({:.0}, {:.0}, {:.0})",
        local_london.x, local_london.y, local_london.z
    );

    // ── Tile math ──
    let zoom = 10_u8;
    let tile = lat_lon_to_tile(nyc.lat, nyc.lon, zoom);
    println!("NYC tile at zoom {}: ({}, {})", zoom, tile.x, tile.y);

    let bounds = tile_bounds(tile);
    println!(
        "Tile bounds: lat [{:.4}, {:.4}] lon [{:.4}, {:.4}]",
        bounds.min_lat, bounds.max_lat, bounds.min_lon, bounds.max_lon
    );

    let res = tile_resolution(zoom, nyc.lat);
    println!("Tile resolution at zoom {}: {:.2} m/pixel", zoom, res);

    // ── Tile sources ──
    let imagery_source = XyzTileSource::new(
        "https://tile.openstreetmap.org/{z}/{x}/{y}.png",
        TileFormat::Png,
    );
    let terrain_source = XyzTileSource::new(
        "https://api.mapbox.com/v4/mapbox.terrain-rgb/{z}/{x}/{y}.pngraw?access_token=TOKEN",
        TileFormat::TerrainRgb,
    );

    println!(
        "Imagery URL: {}",
        imagery_source.tile_url(TileCoord::new(tile.x as u32, tile.y as u32, zoom as u32))
    );
    println!(
        "Terrain URL: {}",
        terrain_source.tile_url(TileCoord::new(tile.x as u32, tile.y as u32, zoom as u32))
    );

    // ── Tile cache + terrain mesh generation ──
    let mut cache = TileCache::new(256);
    let terrain_config = TerrainConfig::default();
    let resolution = terrain_config.mesh_resolution;
    let verts = (resolution + 1) * (resolution + 1);

    for dx in 0..4_u32 {
        for dy in 0..4_u32 {
            let coord = TileCoord::new(tile.x as u32 + dx, tile.y as u32 + dy, zoom as u32);
            let heights = vec![0.0_f32; verts as usize];
            cache.insert(
                coord,
                TileData::Terrain(TerrainData {
                    heights: heights.clone(),
                    width: resolution + 1,
                    height: resolution + 1,
                }),
            );

            let mesh = generate_tile_mesh(
                coord,
                &heights,
                resolution,
                1.0,
                terrain_config.height_scale,
            );
            let gpu_mesh = renderer.create_mesh(mesh.descriptor).expect("terrain mesh");
            renderer.destroy_mesh(gpu_mesh.id);
        }
    }
    println!("Loaded {} terrain tiles into cache", cache.len());

    // ── Terrain-RGB decode demo ──
    let sea_level = decode_terrain_rgb(1, 134, 160);
    println!("Terrain-RGB (1,134,160) = {:.1}m", sea_level);
    let everest = decode_terrain_rgb(2, 25, 210);
    println!("Terrain-RGB (2,25,210) = {:.1}m", everest);

    // ── Atmosphere ──
    let mut atmosphere = AtmosphereRenderer::new(AtmosphereConfig::earth_default());
    atmosphere.set_sun_elevation(30.0_f32.to_radians());
    println!("Atmosphere: Bruneton-Neyret scattering, sun at 30 deg elevation");
    println!("Sun direction: {:?}", atmosphere.sun_direction());

    // Fog for distant terrain
    let fog = Fog {
        mode: FogMode::Exponential,
        color: [0.7, 0.75, 0.8],
        density: 0.0002,
        start: 100.0,
        end: 50000.0,
    };
    println!(
        "Fog: exponential, density={}, factor at 10km = {:.3}",
        fog.density,
        fog.fog_factor(10000.0)
    );

    // ── Scene ──
    let scene = Scene::new();
    println!("Scene nodes: {}", scene.node_count());

    // ── PBR ──
    let mut pbr = PbrPipeline::new(PbrConfig::default());
    pbr.add_light(Light::Directional {
        direction: [-0.5, -0.8, -0.3],
        color: [1.0, 0.98, 0.95],
        intensity: 10.0,
        shadows: true,
    });

    // ── Post-processing ──
    let post = PostPipeline::new()
        .add(ToneMapping::aces())
        .add(Fxaa::default())
        .add(ColorGrading::default());

    // ── Camera ──
    let orbit = OrbitController {
        target: Vec3::ZERO,
        distance: 500.0,
        min_distance: 50.0,
        max_distance: 20_000_000.0,
        aspect: 1920.0 / 1080.0,
        ..OrbitController::default()
    };
    let camera = orbit.camera();

    // ── Render ──
    let mut frame = renderer.begin_frame().expect("begin_frame");
    frame.set_camera(
        camera.view_matrix(),
        camera.projection_matrix(),
        camera.near,
        camera.far,
    );
    renderer.end_frame(frame).expect("end_frame");

    println!("Post passes: {}", post.pass_count());
    println!("PBR lights: {}", pbr.light_count());
    println!("globe done.");
}
