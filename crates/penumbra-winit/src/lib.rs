//! penumbra-winit -- winit windowing integration for Penumbra.
//!
//! Provides [`PenumbraApp`] trait and [`run`] function that creates a winit
//! window and event loop to drive a Penumbra application.

use std::collections::HashSet;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowId};

// ── Window config ──

#[derive(Debug, Clone)]
pub struct WindowConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub vsync: bool,
    pub resizable: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "Penumbra".to_string(),
            width: 1280,
            height: 720,
            vsync: true,
            resizable: true,
        }
    }
}

// ── Key codes ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyCode {
    W,
    A,
    S,
    D,
    Q,
    E,
    Space,
    LShift,
    Escape,
    Up,
    Down,
    Left,
    Right,
}

fn map_key(key: &Key) -> Option<KeyCode> {
    match key {
        Key::Character(c) => match c.as_str() {
            "w" | "W" => Some(KeyCode::W),
            "a" | "A" => Some(KeyCode::A),
            "s" | "S" => Some(KeyCode::S),
            "d" | "D" => Some(KeyCode::D),
            "q" | "Q" => Some(KeyCode::Q),
            "e" | "E" => Some(KeyCode::E),
            " " => Some(KeyCode::Space),
            _ => None,
        },
        Key::Named(NamedKey::Space) => Some(KeyCode::Space),
        Key::Named(NamedKey::Shift) => Some(KeyCode::LShift),
        Key::Named(NamedKey::Escape) => Some(KeyCode::Escape),
        Key::Named(NamedKey::ArrowUp) => Some(KeyCode::Up),
        Key::Named(NamedKey::ArrowDown) => Some(KeyCode::Down),
        Key::Named(NamedKey::ArrowLeft) => Some(KeyCode::Left),
        Key::Named(NamedKey::ArrowRight) => Some(KeyCode::Right),
        _ => None,
    }
}

// ── Input state ──

#[derive(Debug, Clone)]
pub struct InputState {
    pub mouse_position: [f32; 2],
    pub mouse_delta: [f32; 2],
    pub scroll_delta: f32,
    pub mouse_buttons: HashSet<u32>,
    pub keys_pressed: HashSet<KeyCode>,
    pub keys_just_pressed: HashSet<KeyCode>,
    pub keys_just_released: HashSet<KeyCode>,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            mouse_position: [0.0, 0.0],
            mouse_delta: [0.0, 0.0],
            scroll_delta: 0.0,
            mouse_buttons: HashSet::new(),
            keys_pressed: HashSet::new(),
            keys_just_pressed: HashSet::new(),
            keys_just_released: HashSet::new(),
        }
    }

    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.keys_pressed.contains(&key)
    }

    pub fn is_key_just_pressed(&self, key: KeyCode) -> bool {
        self.keys_just_pressed.contains(&key)
    }

    pub fn is_mouse_button_pressed(&self, button: u32) -> bool {
        self.mouse_buttons.contains(&button)
    }

    /// Reset per-frame deltas.
    pub fn end_frame(&mut self) {
        self.mouse_delta = [0.0, 0.0];
        self.scroll_delta = 0.0;
        self.keys_just_pressed.clear();
        self.keys_just_released.clear();
    }
}

impl Default for InputState {
    fn default() -> Self {
        Self::new()
    }
}

// ── PenumbraApp trait ──

/// Application trait. Implement this to build a Penumbra application.
pub trait PenumbraApp: 'static {
    fn init(&mut self);
    fn update(&mut self, dt: f32, input: &InputState);
    fn render(&mut self);
    fn resize(&mut self, width: u32, height: u32);
}

// ── Internal app wrapper for winit ──

struct AppWrapper<A: PenumbraApp> {
    app: A,
    config: WindowConfig,
    window: Option<Window>,
    input: InputState,
    last_frame: std::time::Instant,
}

impl<A: PenumbraApp> ApplicationHandler for AppWrapper<A> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let attrs = Window::default_attributes()
                .with_title(&self.config.title)
                .with_inner_size(winit::dpi::LogicalSize::new(
                    self.config.width,
                    self.config.height,
                ))
                .with_resizable(self.config.resizable);
            match event_loop.create_window(attrs) {
                Ok(window) => {
                    tracing::info!(
                        title = self.config.title,
                        width = self.config.width,
                        height = self.config.height,
                        "Window created"
                    );
                    self.window = Some(window);
                    self.app.init();
                }
                Err(e) => {
                    tracing::error!("Failed to create window: {e}");
                    event_loop.exit();
                }
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                self.app.resize(size.width, size.height);
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let Some(key) = map_key(&event.logical_key) {
                    match event.state {
                        ElementState::Pressed => {
                            if !self.input.keys_pressed.contains(&key) {
                                self.input.keys_just_pressed.insert(key);
                            }
                            self.input.keys_pressed.insert(key);
                        }
                        ElementState::Released => {
                            self.input.keys_pressed.remove(&key);
                            self.input.keys_just_released.insert(key);
                        }
                    }
                }
                // Exit on Escape
                if event.logical_key == Key::Named(NamedKey::Escape) {
                    event_loop.exit();
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                let new_pos = [position.x as f32, position.y as f32];
                self.input.mouse_delta[0] += new_pos[0] - self.input.mouse_position[0];
                self.input.mouse_delta[1] += new_pos[1] - self.input.mouse_position[1];
                self.input.mouse_position = new_pos;
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let btn = match button {
                    winit::event::MouseButton::Left => 0,
                    winit::event::MouseButton::Right => 1,
                    winit::event::MouseButton::Middle => 2,
                    _ => 3,
                };
                match state {
                    ElementState::Pressed => {
                        self.input.mouse_buttons.insert(btn);
                    }
                    ElementState::Released => {
                        self.input.mouse_buttons.remove(&btn);
                    }
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let scroll = match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => y,
                    winit::event::MouseScrollDelta::PixelDelta(pos) => pos.y as f32 * 0.01,
                };
                self.input.scroll_delta += scroll;
            }
            WindowEvent::RedrawRequested => {
                let now = std::time::Instant::now();
                let dt = now.duration_since(self.last_frame).as_secs_f32();
                self.last_frame = now;

                self.app.update(dt, &self.input);
                self.app.render();

                self.input.end_frame();

                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }
}

/// Run a Penumbra application with a winit window and event loop.
pub fn run<A: PenumbraApp>(config: WindowConfig, app: A) {
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let mut wrapper = AppWrapper {
        app,
        config,
        window: None,
        input: InputState::new(),
        last_frame: std::time::Instant::now(),
    };
    event_loop.run_app(&mut wrapper).expect("Event loop error");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_defaults() {
        let cfg = WindowConfig::default();
        assert_eq!(cfg.title, "Penumbra");
        assert_eq!(cfg.width, 1280);
        assert_eq!(cfg.height, 720);
        assert!(cfg.vsync);
        assert!(cfg.resizable);
    }

    #[test]
    fn input_state_key_check() {
        let mut input = InputState::new();
        assert!(!input.is_key_pressed(KeyCode::W));

        input.keys_pressed.insert(KeyCode::W);
        input.keys_just_pressed.insert(KeyCode::W);
        assert!(input.is_key_pressed(KeyCode::W));
        assert!(input.is_key_just_pressed(KeyCode::W));

        input.end_frame();
        assert!(input.is_key_pressed(KeyCode::W)); // still held
        assert!(!input.is_key_just_pressed(KeyCode::W)); // cleared
    }

    #[test]
    fn input_state_mouse() {
        let mut input = InputState::new();
        input.mouse_buttons.insert(0);
        assert!(input.is_mouse_button_pressed(0));
        assert!(!input.is_mouse_button_pressed(1));
    }

    #[test]
    fn key_mapping() {
        assert_eq!(map_key(&Key::Character("w".into())), Some(KeyCode::W));
        assert_eq!(map_key(&Key::Named(NamedKey::Escape)), Some(KeyCode::Escape));
        assert_eq!(map_key(&Key::Named(NamedKey::ArrowUp)), Some(KeyCode::Up));
        assert_eq!(map_key(&Key::Character("z".into())), None);
    }
}
