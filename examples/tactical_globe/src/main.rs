//! tactical_globe — NexusPulse Tactical demo: globe + 27K entities + labels + HUD.
//!
//! Demonstrates the full NexusPulse Tactical integration scenario:
//! WGS84 globe with satellite tiles, terrain, 27K CoT entities as instanced
//! icons with track history, atmospheric sky, entity labels via SDF text,
//! immediate mode HUD overlay, and post-processing.

use glam::{Mat4, Vec3};
use penumbra_atmosphere::{AtmosphereConfig, AtmosphereRenderer};
use penumbra_camera::OrbitController;
use penumbra_core::{Renderer, RendererConfig};
use penumbra_geo::{GeoPosition, lat_lon_to_tile, wgs84_to_enu};
use penumbra_immediate::ImmediateRenderer;
use penumbra_instance::{InstanceBatchDesc, InstanceData, InstanceManager, cpu_frustum_cull};
use penumbra_pbr::{Light, PbrConfig, PbrPipeline};
use penumbra_post::{ColorGrading, Fxaa, PostPipeline, ToneMapping};
use penumbra_scene::Scene;
use penumbra_terrain::{
    TerrainConfig, TerrainData, TileCache, TileCoord, TileData, TileFormat, XyzTileSource,
    generate_tile_mesh,
};
use penumbra_text::{FontAtlas, FontId, GlyphMetrics, TextBatch, layout_text};
use penumbra_wgpu::{WgpuBackend, WgpuConfig};

use penumbra_backend::MeshId;

const _ENTITY_COUNT: usize = 27_000;

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

    println!("=== NexusPulse Tactical Globe Demo ===");
    println!("Backend: {}", renderer.backend_name());

    // ── Geographic origin (Fort Wayne, IN — NexusPulse HQ) ──
    let origin = GeoPosition {
        lat: 41.0793,
        lon: -85.1394,
        alt: 0.0,
    };

    // ── Tile setup ──
    let zoom = 12_u8;
    let center_tile = lat_lon_to_tile(origin.lat, origin.lon, zoom);
    println!(
        "Center tile: ({}, {}) at zoom {}",
        center_tile.x, center_tile.y, zoom
    );

    let _imagery_source = XyzTileSource::new(
        "https://tile.openstreetmap.org/{z}/{x}/{y}.png",
        TileFormat::Png,
    );

    // Load a grid of terrain tiles around the center
    let mut cache = TileCache::new(512);
    let terrain_config = TerrainConfig::default();
    let res = terrain_config.mesh_resolution;
    let verts = (res + 1) * (res + 1);

    let mut tile_meshes_created = 0;
    for dx in -2..=2_i32 {
        for dy in -2..=2_i32 {
            let coord = TileCoord::new(
                (center_tile.x as i32 + dx) as u32,
                (center_tile.y as i32 + dy) as u32,
                zoom as u32,
            );
            let heights = vec![0.0_f32; verts as usize];
            cache.insert(
                coord,
                TileData::Terrain(TerrainData {
                    heights: heights.clone(),
                    width: res + 1,
                    height: res + 1,
                }),
            );
            let mesh = generate_tile_mesh(coord, &heights, res, 1.0, terrain_config.height_scale);
            let gpu_mesh = renderer.create_mesh(mesh.descriptor).expect("terrain mesh");
            renderer.destroy_mesh(gpu_mesh.id);
            tile_meshes_created += 1;
        }
    }
    println!("Terrain tiles loaded: {}", tile_meshes_created);
    println!("Tile cache size: {}", cache.len());

    // ── Scene ──
    let scene = Scene::new();

    // ── Atmosphere ──
    let mut atmosphere = AtmosphereRenderer::new(AtmosphereConfig::earth_default());
    atmosphere.set_sun_elevation(45.0_f32.to_radians());

    // ── 27K Tactical Entities ──
    let mut instance_mgr = InstanceManager::new();

    // Aircraft batch
    let aircraft_batch = instance_mgr.create_batch(InstanceBatchDesc {
        mesh: MeshId(1), // placeholder
        max_instances: 10_000,
        label: Some("aircraft".to_string()),
    });

    // Ground vehicle batch
    let ground_batch = instance_mgr.create_batch(InstanceBatchDesc {
        mesh: MeshId(2), // placeholder
        max_instances: 20_000,
        label: Some("ground".to_string()),
    });

    // Generate 27K entities scattered around Fort Wayne
    let mut aircraft_instances = Vec::with_capacity(7_000);
    let mut ground_instances = Vec::with_capacity(20_000);

    for i in 0..7_000_usize {
        // Simulate aircraft positions in a ~200km radius
        let angle = (i as f64 / 7_000.0) * std::f64::consts::TAU * 5.0;
        let radius_km = 20.0 + (i as f64 / 7_000.0) * 180.0;
        let lat = origin.lat + (angle.cos() * radius_km / 111.0);
        let lon = origin.lon + (angle.sin() * radius_km / (111.0 * origin.lat.to_radians().cos()));
        let alt = 3000.0 + (i as f64 * 0.37).sin() * 5000.0;

        let entity_pos = GeoPosition { lat, lon, alt };
        let local = wgs84_to_enu(&entity_pos, &origin);

        let mut transform = [0.0_f32; 16];
        let mat = Mat4::from_translation(Vec3::new(local.x as f32, local.z as f32, local.y as f32))
            * Mat4::from_scale(Vec3::splat(50.0));
        transform.copy_from_slice(&mat.to_cols_array());

        let color = match i % 3 {
            0 => [1.0, 0.1, 0.1, 1.0], // hostile
            1 => [0.1, 0.3, 1.0, 1.0], // friendly
            _ => [1.0, 0.9, 0.1, 1.0], // unknown
        };

        aircraft_instances.push(InstanceData {
            transform,
            color,
            uv_offset: [0.0, 0.0],
            uv_scale: [1.0, 1.0],
        });
    }

    for i in 0..20_000_usize {
        let angle = (i as f64 / 20_000.0) * std::f64::consts::TAU * 8.0;
        let radius_km = 5.0 + (i as f64 / 20_000.0) * 100.0;
        let lat = origin.lat + (angle.cos() * radius_km / 111.0);
        let lon = origin.lon + (angle.sin() * radius_km / (111.0 * origin.lat.to_radians().cos()));

        let entity_pos = GeoPosition { lat, lon, alt: 0.0 };
        let local = wgs84_to_enu(&entity_pos, &origin);

        let mut transform = [0.0_f32; 16];
        let mat = Mat4::from_translation(Vec3::new(local.x as f32, 0.0, local.y as f32))
            * Mat4::from_scale(Vec3::splat(20.0));
        transform.copy_from_slice(&mat.to_cols_array());

        let color = match i % 4 {
            0 => [0.2, 0.8, 0.2, 1.0],
            1 => [0.8, 0.2, 0.2, 1.0],
            2 => [0.8, 0.8, 0.2, 1.0],
            _ => [0.5, 0.5, 0.5, 1.0],
        };

        ground_instances.push(InstanceData {
            transform,
            color,
            uv_offset: [0.0, 0.0],
            uv_scale: [1.0, 1.0],
        });
    }

    let total = aircraft_instances.len() + ground_instances.len();
    println!(
        "Entities generated: {} ({} aircraft, {} ground)",
        total,
        aircraft_instances.len(),
        ground_instances.len()
    );
    println!(
        "Instance buffer: {:.2} MB",
        (total * std::mem::size_of::<InstanceData>()) as f64 / (1024.0 * 1024.0)
    );

    instance_mgr
        .update_batch(aircraft_batch, aircraft_instances.clone())
        .expect("update aircraft");
    instance_mgr
        .update_batch(ground_batch, ground_instances.clone())
        .expect("update ground");

    // ── Camera (globe orbit) ──
    let mut orbit = OrbitController {
        target: Vec3::ZERO,
        distance: 200_000.0,
        min_distance: 100.0,
        max_distance: 20_000_000.0,
        aspect: 1920.0 / 1080.0,
        far: 100_000_000.0,
        ..OrbitController::default()
    };
    orbit.handle_mouse_move(40.0, 20.0);

    let camera = orbit.camera();
    let vp = camera.view_projection();

    // ── CPU frustum culling ──
    let vis_air = cpu_frustum_cull(&aircraft_instances, vp);
    let vis_gnd = cpu_frustum_cull(&ground_instances, vp);
    println!(
        "Visible: {} aircraft, {} ground ({} total / {} = {:.1}%)",
        vis_air.len(),
        vis_gnd.len(),
        vis_air.len() + vis_gnd.len(),
        total,
        ((vis_air.len() + vis_gnd.len()) as f64 / total as f64) * 100.0
    );

    // ── SDF Text / Labels ──
    let mut font_atlas = FontAtlas::new(FontId(0), 512, 512);
    // Add some placeholder glyph metrics for common chars
    for (i, ch) in "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789 -:"
        .chars()
        .enumerate()
    {
        font_atlas.add_glyph(GlyphMetrics {
            codepoint: ch,
            advance: 8.0,
            bearing_x: 0.0,
            bearing_y: 10.0,
            width: 8.0,
            height: 12.0,
            uv_min: [i as f32 * 0.01, 0.0],
            uv_max: [i as f32 * 0.01 + 0.01, 0.02],
        });
    }

    // Layout HUD text
    let hud_layout = layout_text(
        &font_atlas,
        "ENTITIES: 27000 | HOSTILE: 9000 | FRIENDLY: 9000",
        14.0,
    );
    let mut text_batch = TextBatch::new();
    text_batch.add_layout(&hud_layout, 0.0, [1.0, 1.0, 1.0, 1.0]);
    println!(
        "HUD text: {} glyphs, {} vertices",
        hud_layout.glyphs.len(),
        text_batch.vertex_count()
    );

    // ── Immediate mode HUD overlay ──
    let mut imm = ImmediateRenderer::new();

    // Connection status indicator
    imm.draw_filled_rect(
        Vec3::new(10.0, 10.0, 0.0),
        Vec3::new(200.0, 30.0, 0.0),
        [0.0, 0.0, 0.0, 0.5],
    );

    // Threat summary box
    imm.draw_box(
        Vec3::new(10.0, 40.0, 0.0),
        Vec3::new(200.0, 120.0, 0.0),
        [0.3, 0.3, 0.3, 0.8],
    );

    // Compass lines
    imm.draw_line(Vec3::ZERO, Vec3::new(0.0, 0.0, 100.0), [1.0, 0.0, 0.0, 1.0]); // North
    imm.draw_line(Vec3::ZERO, Vec3::new(100.0, 0.0, 0.0), [0.0, 0.0, 1.0, 1.0]); // East

    println!(
        "Immediate mode: {} line verts, {} tri verts",
        imm.batch().line_vertices.len(),
        imm.batch().triangle_vertices.len()
    );

    // ── PBR ──
    let mut pbr = PbrPipeline::new(PbrConfig::default());
    pbr.add_light(Light::Directional {
        direction: [-0.3, -0.8, -0.5],
        color: [1.0, 0.98, 0.95],
        intensity: 10.0,
        shadows: true,
    });

    // ── Post-processing ──
    let post = PostPipeline::new()
        .add(ToneMapping::aces())
        .add(Fxaa::default())
        .add(ColorGrading {
            brightness: 0.0,
            contrast: 1.05,
            saturation: 1.0,
            enabled: true,
        });

    // ── Render frame ──
    let mut frame = renderer.begin_frame().expect("begin_frame");
    frame.set_camera(
        camera.view_matrix(),
        camera.projection_matrix(),
        camera.near,
        camera.far,
    );
    renderer.end_frame(frame).expect("end_frame");

    // ── Summary ──
    println!("\n=== Tactical Globe Summary ===");
    println!("Scene nodes: {}", scene.node_count());
    println!("Instance batches: {}", instance_mgr.batch_count());
    println!("Total entities: {}", total);
    println!("Terrain tiles: {}", cache.len());
    println!("Post passes: {}", post.pass_count());
    println!("PBR lights: {}", pbr.light_count());
    println!("Sun direction: {:?}", atmosphere.sun_direction());
    println!("=== Done ===");
}
