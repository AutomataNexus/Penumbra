//! Real WASM implementations using wasm-bindgen, web-sys, and js-sys.
//!
//! This module is only compiled on `target_arch = "wasm32"`.

use std::cell::RefCell;
use std::rc::Rc;

use js_sys::Uint8Array;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    Document, HtmlCanvasElement, Navigator, Request, RequestInit, RequestMode, Response, Window,
};

use crate::{BrowserSurface, WebConfig, WebError, WebPlatform};

// ── Helpers ──

fn window() -> Window {
    web_sys::window().expect("no global `window` exists")
}

fn document() -> Document {
    window().document().expect("no `document` on window")
}

fn navigator() -> Navigator {
    window().navigator()
}

// ── Platform detection ──

pub fn detect_platform_impl() -> WebPlatform {
    let win = window();
    let nav = navigator();

    let user_agent = nav.user_agent().unwrap_or_default();
    let pixel_ratio = win.device_pixel_ratio();

    // Check WebGPU support: navigator.gpu exists
    let supports_webgpu = js_sys::Reflect::get(&nav, &JsValue::from_str("gpu"))
        .map(|v| !v.is_undefined() && !v.is_null())
        .unwrap_or(false);

    // Check WebGL2 support: try creating a webgl2 context on a temp canvas
    let supports_webgl2 = {
        let doc = document();
        if let Ok(canvas) = doc.create_element("canvas") {
            canvas
                .dyn_ref::<HtmlCanvasElement>()
                .and_then(|c| c.get_context("webgl2").ok().flatten())
                .is_some()
        } else {
            false
        }
    };

    WebPlatform {
        supports_webgpu,
        supports_webgl2,
        user_agent,
        device_pixel_ratio: pixel_ratio,
        canvas_width: 0,
        canvas_height: 0,
    }
}

// ── Surface creation ──

pub fn create_surface_impl(config: &WebConfig) -> Result<BrowserSurface, WebError> {
    let doc = document();
    let element = doc
        .get_element_by_id(&config.canvas_id)
        .ok_or_else(|| WebError::CanvasNotFound(config.canvas_id.clone()))?;

    let canvas: HtmlCanvasElement = element
        .dyn_into()
        .map_err(|_| WebError::CanvasNotFound(config.canvas_id.clone()))?;

    let pixel_ratio = if config.pixel_ratio > 0.0 {
        config.pixel_ratio
    } else {
        window().device_pixel_ratio()
    };

    let width = (canvas.client_width() as f64 * pixel_ratio) as u32;
    let height = (canvas.client_height() as f64 * pixel_ratio) as u32;
    canvas.set_width(width);
    canvas.set_height(height);

    Ok(BrowserSurface {
        canvas_id: config.canvas_id.clone(),
        width,
        height,
        pixel_ratio,
    })
}

// ── Async tile fetch ──

/// Fetch a single tile using the Fetch API. Must be called from within a WASM async context.
///
/// Usage: `wasm_bindgen_futures::spawn_local(async { let data = fetch_tile_async(url).await; })`
pub async fn fetch_tile_async(url: &str) -> Result<Vec<u8>, WebError> {
    let mut opts = RequestInit::new();
    opts.method("GET");
    opts.mode(RequestMode::Cors);

    let request = Request::new_with_str_and_init(url, &opts)
        .map_err(|e| WebError::FetchFailed(format!("{e:?}")))?;

    let resp_value = JsFuture::from(window().fetch_with_request(&request))
        .await
        .map_err(|e| WebError::FetchFailed(format!("{e:?}")))?;

    let resp: Response = resp_value
        .dyn_into()
        .map_err(|_| WebError::FetchFailed("Response cast failed".to_string()))?;

    if !resp.ok() {
        return Err(WebError::FetchFailed(format!(
            "HTTP {} {}",
            resp.status(),
            resp.status_text()
        )));
    }

    let array_buffer = JsFuture::from(
        resp.array_buffer()
            .map_err(|e| WebError::FetchFailed(format!("{e:?}")))?,
    )
    .await
    .map_err(|e| WebError::FetchFailed(format!("{e:?}")))?;

    let uint8_array = Uint8Array::new(&array_buffer);
    Ok(uint8_array.to_vec())
}

// ── Render loop via requestAnimationFrame ──

pub fn run_loop_impl<F: FnMut(f32) + 'static>(callback: F) {
    let callback = Rc::new(RefCell::new(callback));
    let last_time = Rc::new(RefCell::new(None::<f64>));

    // Recursive requestAnimationFrame
    let f: Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>> = Rc::new(RefCell::new(None));
    let g = f.clone();

    let cb = callback.clone();
    let lt = last_time.clone();

    *g.borrow_mut() = Some(Closure::new(move |timestamp: f64| {
        let dt = {
            let mut lt = lt.borrow_mut();
            let dt = lt.map_or(0.0, |prev| (timestamp - prev) / 1000.0);
            *lt = Some(timestamp);
            dt as f32
        };

        (cb.borrow_mut())(dt);

        // Schedule next frame
        window()
            .request_animation_frame(f.borrow().as_ref().unwrap().as_ref().unchecked_ref())
            .expect("request_animation_frame failed");
    }));

    // Kick off the first frame
    window()
        .request_animation_frame(g.borrow().as_ref().unwrap().as_ref().unchecked_ref())
        .expect("request_animation_frame failed");
}

// ── Init ──

pub fn init_wasm_impl() {
    console_error_panic_hook::set_once();
    tracing::info!("penumbra-web WASM environment initialized");
}
