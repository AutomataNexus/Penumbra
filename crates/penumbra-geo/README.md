# penumbra-geo

Geospatial utilities for the Penumbra 3D rendering SDK.

## Purpose

penumbra-geo provides WGS84 geodesy, coordinate conversions, and tile math
required to place 3D content on or near the Earth's surface. It sits between
raw GPS/GIS data and the renderer, converting geographic coordinates into the
local Cartesian frames that penumbra-scene and penumbra-terrain consume.

## Key types

| Type | Description |
|------|-------------|
| `GeoPosition` | Latitude, longitude (degrees) and altitude (meters) on the WGS84 ellipsoid |
| `TileCoord` | Web Mercator tile address (x, y, zoom) |
| `GeoBounds` | Axis-aligned bounding box in lat/lon space |

## Usage example

```rust
use penumbra_geo::{GeoPosition, wgs84_to_ecef, haversine_distance, lat_lon_to_tile};

let paris = GeoPosition::new(48.8566, 2.3522, 0.0);
let london = GeoPosition::new(51.5074, -0.1278, 0.0);

// Distance in meters
let d = haversine_distance(&paris, &london);
println!("Paris -> London: {d:.0} m");

// ECEF coordinates
let ecef = wgs84_to_ecef(&paris);
println!("Paris ECEF: {ecef}");

// Tile at zoom 10
let tile = lat_lon_to_tile(paris.lat, paris.lon, 10);
println!("Tile: x={}, y={}, z={}", tile.x, tile.y, tile.zoom);
```

## Running tests

```bash
cargo test -p penumbra-geo
```
