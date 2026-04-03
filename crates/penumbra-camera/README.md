# penumbra-camera

Camera system for the **Penumbra** 3D rendering SDK.

## Purpose

`penumbra-camera` supplies perspective and orthographic camera abstractions,
interactive orbit and fly controllers, and ray-casting utilities used throughout
the Penumbra rendering pipeline. It sits between user input handling
(`penumbra-winit`) and the rendering crates (`penumbra-pbr`, `penumbra-scene`)
that consume view and projection matrices.

## Key types

| Type | Description |
|---|---|
| `PerspectiveCamera` | Position/target camera with perspective projection |
| `OrthographicCamera` | Position/target camera with orthographic projection |
| `Camera` | Enum wrapping either projection type |
| `OrbitController` | Rotate around a target point with mouse + scroll |
| `FlyController` | WASD + mouse-look first-person controller |
| `Ray` | Origin + direction with plane and AABB intersection |
| `screen_to_ray` | Convert a screen pixel to a world-space ray |

## Usage example

```rust
use penumbra_camera::{PerspectiveCamera, OrbitController, screen_to_ray, Ray};
use glam::{Vec2, Vec3};

// Create an orbit controller and get a camera from it.
let mut orbit = OrbitController::default();
orbit.handle_mouse_move(10.0, -5.0);
orbit.handle_scroll(1.0);
let cam = orbit.camera();

// Compute view-projection matrix.
let vp = cam.view_projection();

// Cast a ray from the center of the screen.
let inv_vp = vp.inverse();
let ray = screen_to_ray(Vec2::new(400.0, 300.0), Vec2::new(800.0, 600.0), inv_vp);
let hit = ray.intersect_plane(Vec3::Y, 0.0);
```

## Running tests

```sh
cargo test -p penumbra-camera
```
