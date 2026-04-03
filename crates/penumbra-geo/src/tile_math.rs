//! Web Mercator tile math.

use serde::{Deserialize, Serialize};

use crate::position::GeoPosition;

/// A tile address in the Web Mercator tiling scheme (e.g. slippy map tiles).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TileCoord {
    pub x: u32,
    pub y: u32,
    pub zoom: u8,
}

/// An axis-aligned bounding box in geographic (lat/lon) space.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GeoBounds {
    pub min_lat: f64,
    pub max_lat: f64,
    pub min_lon: f64,
    pub max_lon: f64,
}

/// Convert latitude/longitude (degrees) to a tile coordinate at the given zoom level.
///
/// Uses the standard Web Mercator (EPSG:3857) formulas.
pub fn lat_lon_to_tile(lat: f64, lon: f64, zoom: u8) -> TileCoord {
    // Web Mercator is only defined for ~±85.051 degrees latitude.
    let lat = lat.clamp(-85.0511, 85.0511);
    let n = (1u64 << zoom) as f64;
    let max_tile = n as u32 - 1;
    let lat_rad = lat.to_radians();

    let x = (((lon + 180.0) / 360.0 * n).floor() as u32).min(max_tile);
    let y_raw = (1.0 - (lat_rad.tan() + 1.0 / lat_rad.cos()).ln() / std::f64::consts::PI) / 2.0 * n;
    let y = (y_raw.floor().max(0.0) as u32).min(max_tile);

    TileCoord { x, y, zoom }
}

/// Compute the geographic bounding box of a tile.
pub fn tile_bounds(coord: TileCoord) -> GeoBounds {
    let n = (1u64 << coord.zoom) as f64;

    let min_lon = coord.x as f64 / n * 360.0 - 180.0;
    let max_lon = (coord.x + 1) as f64 / n * 360.0 - 180.0;

    // Note: y=0 is the top (north), y increases southward.
    let max_lat = tile_y_to_lat(coord.y, n);
    let min_lat = tile_y_to_lat(coord.y + 1, n);

    GeoBounds {
        min_lat,
        max_lat,
        min_lon,
        max_lon,
    }
}

/// Compute the ground resolution in meters per pixel at a given zoom level and latitude.
///
/// Assumes 256-pixel tiles.
pub fn tile_resolution(zoom: u8, lat: f64) -> f64 {
    let circumference = 2.0 * std::f64::consts::PI * 6_378_137.0; // WGS84 semi-major axis
    let total_pixels = 256.0 * (1u64 << zoom) as f64;
    circumference * lat.to_radians().cos() / total_pixels
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn tile_y_to_lat(y: u32, n: f64) -> f64 {
    let val = std::f64::consts::PI * (1.0 - 2.0 * y as f64 / n);
    val.sinh().atan().to_degrees()
}

/// Convert a [`GeoBounds`] center to a [`GeoPosition`] at zero altitude.
impl GeoBounds {
    /// Return the center of the bounds as a [`GeoPosition`] at altitude 0.
    pub fn center(&self) -> GeoPosition {
        GeoPosition::new(
            (self.min_lat + self.max_lat) / 2.0,
            (self.min_lon + self.max_lon) / 2.0,
            0.0,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tile_at_zoom_0() {
        let tile = lat_lon_to_tile(0.0, 0.0, 0);
        assert_eq!(tile, TileCoord { x: 0, y: 0, zoom: 0 });
    }

    #[test]
    fn tile_known_values_zoom_2() {
        // Zoom 2: 4x4 grid. (0, 0) should be in tile (2, 1) roughly.
        let tile = lat_lon_to_tile(0.0, 0.0, 2);
        assert_eq!(tile.x, 2);
        assert_eq!(tile.y, 2);
        assert_eq!(tile.zoom, 2);
    }

    #[test]
    fn tile_new_york_zoom_10() {
        // New York City (40.7128, -74.0060)
        let tile = lat_lon_to_tile(40.7128, -74.0060, 10);
        assert_eq!(tile.x, 301);
        assert_eq!(tile.y, 385);
        assert_eq!(tile.zoom, 10);
    }

    #[test]
    fn tile_bounds_round_trip() {
        let zoom: u8 = 10;
        let lat = 48.8566;
        let lon = 2.3522;
        let tile = lat_lon_to_tile(lat, lon, zoom);
        let bounds = tile_bounds(tile);

        // The original point should be inside the bounds.
        assert!(lat >= bounds.min_lat && lat <= bounds.max_lat);
        assert!(lon >= bounds.min_lon && lon <= bounds.max_lon);
    }

    #[test]
    fn tile_bounds_zoom_0_covers_world() {
        let bounds = tile_bounds(TileCoord { x: 0, y: 0, zoom: 0 });
        assert!((bounds.min_lon - (-180.0)).abs() < 1e-6);
        assert!((bounds.max_lon - 180.0).abs() < 1e-6);
        // Web Mercator doesn't cover full +-90 but should be close to ~85.05
        assert!(bounds.max_lat > 85.0);
        assert!(bounds.min_lat < -85.0);
    }

    #[test]
    fn tile_resolution_equator_zoom_0() {
        let res = tile_resolution(0, 0.0);
        // At zoom 0, one 256px tile covers the whole world.
        // Circumference / 256 ~ 156543 m/px
        let expected = 2.0 * std::f64::consts::PI * 6_378_137.0 / 256.0;
        assert!(
            (res - expected).abs() < 1.0,
            "expected ~{expected}, got {res}"
        );
    }

    #[test]
    fn tile_resolution_decreases_with_zoom() {
        let r0 = tile_resolution(0, 45.0);
        let r1 = tile_resolution(1, 45.0);
        assert!(r0 > r1 * 1.99);
        assert!(r0 < r1 * 2.01);
    }

    #[test]
    fn tile_resolution_decreases_toward_pole() {
        let eq = tile_resolution(10, 0.0);
        let mid = tile_resolution(10, 60.0);
        assert!(mid < eq, "resolution should decrease at higher latitudes");
        assert!(
            (mid - eq * 0.5).abs() < eq * 0.01,
            "at 60 deg should be ~half equatorial"
        );
    }
}
