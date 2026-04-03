//! Windowed rendering example — first visible pixels through the Penumbra pipeline.
//!
//! Opens a window, creates a cube mesh through the SDK, renders it using the
//! default PBR pipeline with orbit camera.

use std::sync::Arc;

use glam::{Mat4, Vec3};
use penumbra_asset::cube_mesh;
use penumbra_camera::OrbitController;
use penumbra_core::{DrawCall, Material, MaterialId, Renderer, RendererConfig, Rgba};
use penumbra_wgpu::{WgpuBackend, WgpuConfig};
use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowId};

struct App {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    cube_mesh_id: Option<penumbra_backend::MeshId>,
    material_id: Option<MaterialId>,
    camera: OrbitController,
    mouse_pressed: bool,
    last_mouse: [f32; 2],
    time: f32,
}

impl App {
    fn new() -> Self {
        Self {
            window: None,
            renderer: None,
            cube_mesh_id: None,
            material_id: None,
            camera: OrbitController {
                target: Vec3::ZERO,
                distance: 4.0,
                phi: 1.0,
                theta: 0.5,
                aspect: 16.0 / 9.0,
                ..OrbitController::default()
            },
            mouse_pressed: false,
            last_mouse: [0.0; 2],
            time: 0.0,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let attrs = Window::default_attributes()
            .with_title("Penumbra — Windowed Rendering")
            .with_inner_size(winit::dpi::LogicalSize::new(1280, 720));
        let window = Arc::new(event_loop.create_window(attrs).expect("create window"));

        // Create the wgpu backend with a real surface
        let backend =
            WgpuBackend::from_window(window.clone(), WgpuConfig::default()).expect("wgpu backend");

        let size = window.inner_size();
        let mut renderer = Renderer::new(
            backend,
            RendererConfig {
                width: size.width,
                height: size.height,
                ..RendererConfig::default()
            },
        );

        // Initialize the default PBR pipeline
        renderer.init_pipeline().expect("init pipeline");

        // Create a cube mesh through the SDK
        let gpu_mesh = renderer.create_mesh(cube_mesh()).expect("create mesh");
        self.cube_mesh_id = Some(gpu_mesh.id);

        // Create a material
        let mat_id = renderer.add_material(Material {
            albedo: Rgba::new(0.8, 0.15, 0.1, 1.0),
            metallic: 0.6,
            roughness: 0.4,
            ..Material::default()
        });
        self.material_id = Some(mat_id);

        self.renderer = Some(renderer);
        self.window = Some(window);

        println!("Window opened — rendering through Penumbra SDK pipeline");
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::KeyboardInput { event, .. }
                if event.logical_key == Key::Named(NamedKey::Escape) =>
            {
                event_loop.exit()
            }
            WindowEvent::Resized(size) => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(size.width, size.height);
                    self.camera.aspect = size.width as f32 / size.height as f32;
                }
            }
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => {
                self.mouse_pressed = state == ElementState::Pressed;
            }
            WindowEvent::CursorMoved { position, .. } => {
                let pos = [position.x as f32, position.y as f32];
                if self.mouse_pressed {
                    let dx = pos[0] - self.last_mouse[0];
                    let dy = pos[1] - self.last_mouse[1];
                    self.camera.handle_mouse_move(dx, dy);
                }
                self.last_mouse = pos;
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let scroll = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y,
                    MouseScrollDelta::PixelDelta(p) => p.y as f32 * 0.01,
                };
                self.camera.handle_scroll(scroll);
            }
            WindowEvent::RedrawRequested => {
                self.time += 1.0 / 60.0;

                // Auto-rotate when not dragging
                if !self.mouse_pressed {
                    self.camera.theta += 0.003;
                }

                if let (Some(renderer), Some(mesh_id), Some(mat_id)) =
                    (&mut self.renderer, &self.cube_mesh_id, &self.material_id)
                {
                    // Get camera matrices
                    let camera = self.camera.camera();
                    let view = camera.view_matrix();
                    let proj = camera.projection_matrix();

                    // Begin frame
                    let mut frame = match renderer.begin_frame() {
                        Ok(f) => f,
                        Err(_) => return,
                    };
                    frame.set_camera(view, proj, camera.near, camera.far);

                    // Submit a draw call for the cube
                    let pipeline_id = penumbra_backend::PipelineId(0); // placeholder
                    let transform =
                        Mat4::from_rotation_y(self.time * 0.5) * Mat4::from_rotation_x(0.3);
                    frame.submit(DrawCall::new(*mesh_id, *mat_id, pipeline_id, transform));

                    // End frame — this executes the draw calls through the GPU
                    if let Err(e) = renderer.end_frame(frame) {
                        tracing::error!("end_frame error: {e}");
                    }
                }

                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }
}

fn main() {
    tracing_subscriber::fmt::init();
    println!("Penumbra Windowed Example");
    println!("Controls: Left drag = orbit, Scroll = zoom, Escape = quit\n");
    let event_loop = EventLoop::new().expect("event loop");
    event_loop.run_app(&mut App::new()).expect("run");
}
