//! Core geographic position type.

use serde::{Deserialize, Serialize};

/// A position on the WGS84 ellipsoid.
///
/// - `lat` and `lon` are in **degrees** (geodetic latitude, longitude).
/// - `alt` is the height above the ellipsoid in **meters**.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GeoPosition {
    /// Geodetic latitude in degrees. Positive north, negative south.
    pub lat: f64,
    /// Geodetic longitude in degrees. Positive east, negative west.
    pub lon: f64,
    /// Height above the WGS84 ellipsoid in meters.
    pub alt: f64,
}

impl GeoPosition {
    /// Create a new [`GeoPosition`].
    #[inline]
    pub fn new(lat: f64, lon: f64, alt: f64) -> Self {
        Self { lat, lon, alt }
    }
}
