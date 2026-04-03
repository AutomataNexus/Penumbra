# penumbra-backend

The GPU abstraction layer for Penumbra. Defines the `RenderBackend` trait that all feature crates depend on.

## Position in Pipeline

```
Feature Crates (penumbra-pbr, penumbra-terrain, etc.)
    |
    v
penumbra-backend  <-- RenderBackend trait defined here
    |
    v
penumbra-wgpu (or your custom backend)
```

## Key Public Types

- `RenderBackend` — trait that abstracts over GPU APIs
- `BackendCapabilities` — reports what the backend supports
- `MeshDescriptor` / `TextureDescriptor` / `BufferDescriptor` — resource creation descriptors
- `RenderPassDescriptor` / `RenderPassHandle` — render pass management
- `BackendError` — error type for all backend operations

## Usage

```rust
use penumbra_backend::{RenderBackend, BackendCapabilities};

fn render(backend: &mut dyn RenderBackend) {
    backend.begin_frame().unwrap();
    let pass = backend.begin_render_pass(desc);
    backend.set_pipeline(pass, pipeline_id);
    backend.draw_indexed(pass, 0..36, 0, 0..1);
    backend.end_render_pass(pass);
    backend.end_frame().unwrap();
    backend.present().unwrap();
}
```

## Tests

```bash
cargo test -p penumbra-backend
```
