//! penumbra-wasm-example -- Browser WASM example for Penumbra.
//!
//! Renders a PBR-lit scene in the browser using WebGPU or WebGL2.
//! Build with: `wasm-pack build examples/wasm --target web`
//! Serve with any static HTTP server and open index.html.

use glam::{Quat, Vec3};
use penumbra_camera::OrbitController;
#[cfg(target_arch = "wasm32")]
use penumbra_geo::{GeoPosition, haversine_distance};
use penumbra_immediate::ImmediateRenderer;
use penumbra_pbr::{Light, PbrConfig, PbrPipeline};
use penumbra_scene::{Scene, Transform};
#[cfg(target_arch = "wasm32")]
use penumbra_web::WebConfig;
use penumbra_web::detect_platform;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

/// Application state for the WASM example.
#[allow(dead_code)]
struct App {
    scene: Scene,
    orbit: OrbitController,
    pbr: PbrPipeline,
    immediate: ImmediateRenderer,
    frame_count: u64,
}

impl App {
    fn new() -> Self {
        // Scene graph with a cube node
        let mut scene = Scene::new();
        let cube = scene.add_mesh(penumbra_backend::MeshId(1), penumbra_core::MaterialId(1));
        scene.set_transform(
            cube,
            Transform {
                translation: Vec3::ZERO,
                rotation: Quat::from_euler(glam::EulerRot::YXZ, 0.4, 0.3, 0.0),
                scale: Vec3::ONE,
            },
        );

        // Camera
        let orbit = OrbitController {
            target: Vec3::ZERO,
            distance: 5.0,
            aspect: 16.0 / 9.0,
            ..OrbitController::default()
        };

        // PBR lighting
        let mut pbr = PbrPipeline::new(PbrConfig::default());
        pbr.add_light(Light::Directional {
            direction: [-0.5, -1.0, -0.3],
            color: [1.0, 0.98, 0.95],
            intensity: 10.0,
            shadows: true,
        });
        pbr.add_light(Light::Point {
            position: [3.0, 2.0, 4.0],
            color: [0.4, 0.5, 0.8],
            intensity: 5.0,
            range: 20.0,
            shadows: false,
        });

        // Immediate mode for debug overlays
        let immediate = ImmediateRenderer::new();

        Self {
            scene,
            orbit,
            pbr,
            immediate,
            frame_count: 0,
        }
    }

    fn update(&mut self, dt: f32) {
        // Slowly rotate the camera
        self.orbit.handle_mouse_move(dt * 20.0, 0.0);

        // Update scene transforms
        self.scene.update_transforms();

        // Draw debug grid
        self.immediate.clear();
        self.immediate.draw_grid(1.0, 10, [0.3, 0.3, 0.3, 0.5]);
        self.immediate
            .draw_line(Vec3::ZERO, Vec3::new(2.0, 0.0, 0.0), [1.0, 0.0, 0.0, 1.0]);
        self.immediate
            .draw_line(Vec3::ZERO, Vec3::new(0.0, 2.0, 0.0), [0.0, 1.0, 0.0, 1.0]);
        self.immediate
            .draw_line(Vec3::ZERO, Vec3::new(0.0, 0.0, 2.0), [0.0, 0.0, 1.0, 1.0]);

        self.frame_count += 1;
    }
}

/// Entry point for the WASM example.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();

    tracing::info!("Penumbra WASM example starting");

    // Detect platform
    let platform = detect_platform();
    tracing::info!(
        webgpu = platform.supports_webgpu,
        webgl2 = platform.supports_webgl2,
        dpi = platform.device_pixel_ratio,
        ua = platform.user_agent,
        "Platform detected"
    );

    // Create surface
    let config = WebConfig::default();
    let surface = penumbra_web::create_surface(&config);
    match &surface {
        Ok(s) => tracing::info!(w = s.width, h = s.height, "Surface created"),
        Err(e) => tracing::error!("Surface creation failed: {e}"),
    }

    // Demo: geodesy works in WASM
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
    let dist = haversine_distance(&nyc, &london);
    tracing::info!(km = dist / 1000.0, "NYC -> London distance");

    // Create app and start render loop
    let mut app = App::new();
    penumbra_web::run_loop(move |dt| {
        app.update(dt);
    });
}

/// Non-WASM entry for testing compilation.
#[cfg(not(target_arch = "wasm32"))]
pub fn main() {
    tracing_subscriber::fmt::init();
    println!("Penumbra WASM example (native stub)");
    println!("Build for WASM with: wasm-pack build examples/wasm --target web");

    let platform = detect_platform();
    println!(
        "Platform: webgpu={}, webgl2={}",
        platform.supports_webgpu, platform.supports_webgl2
    );

    let mut app = App::new();
    app.update(0.016);
    println!(
        "Frame {} rendered, {} immediate lines",
        app.frame_count,
        app.immediate.batch().line_vertices.len()
    );
}
