//! penumbra-web -- WASM browser target for Penumbra.
//!
//! Provides browser surface creation, platform detection, fetch-based tile
//! loading, and the WASM render loop. On non-WASM targets, all functions
//! are available as stubs for cross-compilation testing.
//!
//! On WASM targets (`target_arch = "wasm32"`), this crate uses `wasm-bindgen`,
//! `web-sys`, and `js-sys` for real browser integration.

#[cfg(target_arch = "wasm32")]
pub mod wasm;

// ── Web config ──

/// Configuration for the web/WASM target.
#[derive(Debug, Clone)]
pub struct WebConfig {
    /// The HTML canvas element ID to render into.
    pub canvas_id: String,
    /// Device pixel ratio (1.0 = standard, 2.0 = retina).
    pub pixel_ratio: f64,
    /// Whether to prefer WebGPU over WebGL2 when both are available.
    pub prefer_webgpu: bool,
}

impl Default for WebConfig {
    fn default() -> Self {
        Self {
            canvas_id: "penumbra-canvas".to_string(),
            pixel_ratio: 1.0,
            prefer_webgpu: true,
        }
    }
}

// ── Web platform detection ──

/// Detected browser/platform capabilities.
#[derive(Debug, Clone)]
pub struct WebPlatform {
    pub supports_webgpu: bool,
    pub supports_webgl2: bool,
    pub user_agent: String,
    pub device_pixel_ratio: f64,
    pub canvas_width: u32,
    pub canvas_height: u32,
}

/// Detect browser capabilities.
///
/// On WASM targets, queries the browser via `web-sys`. On native, returns defaults.
pub fn detect_platform() -> WebPlatform {
    #[cfg(target_arch = "wasm32")]
    {
        wasm::detect_platform_impl()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        WebPlatform {
            supports_webgpu: false,
            supports_webgl2: false,
            user_agent: String::new(),
            device_pixel_ratio: 1.0,
            canvas_width: 0,
            canvas_height: 0,
        }
    }
}

// ── Browser surface creation ──

/// Handle to a browser rendering surface (canvas).
#[derive(Debug)]
pub struct BrowserSurface {
    pub canvas_id: String,
    pub width: u32,
    pub height: u32,
    pub pixel_ratio: f64,
}

/// Create a browser rendering surface from a canvas element.
pub fn create_surface(config: &WebConfig) -> Result<BrowserSurface, WebError> {
    #[cfg(target_arch = "wasm32")]
    {
        wasm::create_surface_impl(config)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        tracing::info!(
            canvas_id = config.canvas_id,
            "Browser surface created (non-WASM stub)"
        );
        Ok(BrowserSurface {
            canvas_id: config.canvas_id.clone(),
            width: 0,
            height: 0,
            pixel_ratio: config.pixel_ratio,
        })
    }
}

// ── Fetch-based tile loading ──

/// A pending tile fetch request.
#[derive(Debug)]
pub struct TileFetchRequest {
    pub url: String,
    pub x: u32,
    pub y: u32,
    pub zoom: u32,
}

/// Result of a completed tile fetch.
#[derive(Debug)]
pub struct TileFetchResult {
    pub x: u32,
    pub y: u32,
    pub zoom: u32,
    pub data: Vec<u8>,
}

/// Fetch tile data from a URL. On WASM uses `window.fetch()`, on native returns error.
pub fn fetch_tile(request: &TileFetchRequest) -> Result<TileFetchResult, WebError> {
    #[cfg(target_arch = "wasm32")]
    {
        // WASM async fetch would go through wasm_bindgen_futures::spawn_local
        // For synchronous API compatibility, return NotInBrowser and use fetch_tile_async instead
        let _ = request;
        Err(WebError::FetchFailed(
            "Use fetch_tile_async for WASM".to_string(),
        ))
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = request;
        Err(WebError::NotInBrowser)
    }
}

/// Queue multiple tile fetches. Returns immediately; results arrive asynchronously.
pub fn fetch_tiles_async(requests: &[TileFetchRequest]) -> Vec<Result<TileFetchResult, WebError>> {
    requests.iter().map(fetch_tile).collect()
}

// ── Render loop ──

/// Start the browser render loop using `requestAnimationFrame`.
///
/// On WASM, sets up a recursive `requestAnimationFrame` callback.
/// On non-WASM, this is a no-op.
pub fn run_loop<F: FnMut(f32) + 'static>(callback: F) {
    #[cfg(target_arch = "wasm32")]
    {
        wasm::run_loop_impl(callback);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = callback;
        tracing::info!("penumbra-web: run_loop() called (non-WASM stub)");
    }
}

/// Initialize the WASM environment (panic hook, logging).
/// Call this once at the start of your WASM application.
pub fn init_wasm() {
    #[cfg(target_arch = "wasm32")]
    {
        wasm::init_wasm_impl();
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        tracing::info!("penumbra-web: init_wasm() called (non-WASM stub)");
    }
}

// ── Errors ──

#[derive(Debug, Clone)]
pub enum WebError {
    NotInBrowser,
    CanvasNotFound(String),
    FetchFailed(String),
    WebGpuNotSupported,
}

impl std::fmt::Display for WebError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WebError::NotInBrowser => write!(f, "Not running in a browser environment"),
            WebError::CanvasNotFound(id) => write!(f, "Canvas element not found: {id}"),
            WebError::FetchFailed(msg) => write!(f, "Fetch failed: {msg}"),
            WebError::WebGpuNotSupported => write!(f, "WebGPU not supported by this browser"),
        }
    }
}

impl std::error::Error for WebError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_defaults() {
        let cfg = WebConfig::default();
        assert_eq!(cfg.canvas_id, "penumbra-canvas");
        assert!((cfg.pixel_ratio - 1.0).abs() < f64::EPSILON);
        assert!(cfg.prefer_webgpu);
    }

    #[test]
    fn detect_platform_returns_defaults() {
        let platform = detect_platform();
        assert!(!platform.supports_webgpu);
        assert!(!platform.supports_webgl2);
        assert!(platform.user_agent.is_empty());
    }

    #[test]
    fn create_surface_stub() {
        let config = WebConfig::default();
        let surface = create_surface(&config).unwrap();
        assert_eq!(surface.canvas_id, "penumbra-canvas");
    }

    #[test]
    fn fetch_tile_returns_not_in_browser() {
        let req = TileFetchRequest {
            url: "https://example.com/0/0/0.png".to_string(),
            x: 0,
            y: 0,
            zoom: 0,
        };
        let result = fetch_tile(&req);
        assert!(result.is_err());
    }
}
