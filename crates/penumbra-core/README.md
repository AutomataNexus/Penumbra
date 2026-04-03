# penumbra-core

Core types for the Penumbra 3D rendering SDK. Owns the `Renderer`, `RenderFrame`, and all shared types.

## Position in Pipeline

```
Your Application
    |
    v
penumbra-core  <-- Renderer, RenderFrame, Material, DrawCall
    |
    v
penumbra-backend (RenderBackend trait)
```

## Key Public Types

- `Renderer` — central renderer, owns the backend, manages frame lifecycle
- `RenderFrame` — a single frame being rendered, collects draw calls
- `RendererConfig` — renderer configuration (resolution, MSAA, HDR, vsync)
- `FrameStats` — per-frame performance statistics
- `Material` — PBR material parameters and texture references

## Usage

```rust
use penumbra_core::{Renderer, RendererConfig, RenderFrame};

let mut renderer = Renderer::new(backend, RendererConfig::default());
let mut frame = renderer.begin_frame();
// submit draw calls to frame...
renderer.end_frame(frame);
```

## Tests

```bash
cargo test -p penumbra-core
```
