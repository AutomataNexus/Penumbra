//! Coordinate conversions between WGS84, ECEF, and ENU frames.

use glam::DVec3;

use crate::position::GeoPosition;

// ---------------------------------------------------------------------------
// WGS84 ellipsoid constants
// ---------------------------------------------------------------------------

/// Semi-major axis in meters.
pub const WGS84_A: f64 = 6_378_137.0;

/// Flattening.
pub const WGS84_F: f64 = 1.0 / 298.257_223_563;

/// Semi-minor axis in meters: b = a * (1 - f).
pub const WGS84_B: f64 = WGS84_A * (1.0 - WGS84_F);

/// First eccentricity squared: e² = 2f - f².
pub const WGS84_E2: f64 = 2.0 * WGS84_F - WGS84_F * WGS84_F;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Radius of curvature in the prime vertical.
#[inline]
fn prime_vertical_radius(sin_lat: f64) -> f64 {
    WGS84_A / (1.0 - WGS84_E2 * sin_lat * sin_lat).sqrt()
}

// ---------------------------------------------------------------------------
// WGS84 <-> ECEF
// ---------------------------------------------------------------------------

/// Convert a WGS84 position to Earth-Centered Earth-Fixed (ECEF) coordinates.
pub fn wgs84_to_ecef(pos: &GeoPosition) -> DVec3 {
    let lat = pos.lat.to_radians();
    let lon = pos.lon.to_radians();
    let (sin_lat, cos_lat) = lat.sin_cos();
    let (sin_lon, cos_lon) = lon.sin_cos();
    let n = prime_vertical_radius(sin_lat);
    let h = pos.alt;

    DVec3::new(
        (n + h) * cos_lat * cos_lon,
        (n + h) * cos_lat * sin_lon,
        (n * (1.0 - WGS84_E2) + h) * sin_lat,
    )
}

/// Convert ECEF coordinates back to WGS84 (iterative Bowring method).
pub fn ecef_to_wgs84(ecef: DVec3) -> GeoPosition {
    let x = ecef.x;
    let y = ecef.y;
    let z = ecef.z;

    let lon = y.atan2(x);
    let p = (x * x + y * y).sqrt();

    // Iterative solution for latitude and altitude (Bowring)
    let mut lat = (z / (p * (1.0 - WGS84_E2))).atan(); // initial estimate
    for _ in 0..10 {
        let sin_lat = lat.sin();
        let n = prime_vertical_radius(sin_lat);
        lat = (z + WGS84_E2 * n * sin_lat).atan2(p);
    }

    let sin_lat = lat.sin();
    let cos_lat = lat.cos();
    let n = prime_vertical_radius(sin_lat);

    let alt = if cos_lat.abs() > 1e-10 {
        p / cos_lat - n
    } else {
        z.abs() / sin_lat.abs() - n * (1.0 - WGS84_E2)
    };

    GeoPosition::new(lat.to_degrees(), lon.to_degrees(), alt)
}

// ---------------------------------------------------------------------------
// WGS84 <-> ENU (local tangent plane)
// ---------------------------------------------------------------------------

/// Convert a WGS84 position to a local East-North-Up frame centred at `origin`.
pub fn wgs84_to_enu(position: &GeoPosition, origin: &GeoPosition) -> DVec3 {
    let pos_ecef = wgs84_to_ecef(position);
    let org_ecef = wgs84_to_ecef(origin);
    let diff = pos_ecef - org_ecef;

    let lat = origin.lat.to_radians();
    let lon = origin.lon.to_radians();
    let (sin_lat, cos_lat) = lat.sin_cos();
    let (sin_lon, cos_lon) = lon.sin_cos();

    // Rotation from ECEF to ENU
    let east = -sin_lon * diff.x + cos_lon * diff.y;
    let north = -sin_lat * cos_lon * diff.x - sin_lat * sin_lon * diff.y + cos_lat * diff.z;
    let up = cos_lat * cos_lon * diff.x + cos_lat * sin_lon * diff.y + sin_lat * diff.z;

    DVec3::new(east, north, up)
}

/// Convert an ENU vector (relative to `origin`) back to a WGS84 position.
pub fn enu_to_wgs84(enu: DVec3, origin: &GeoPosition) -> GeoPosition {
    let lat = origin.lat.to_radians();
    let lon = origin.lon.to_radians();
    let (sin_lat, cos_lat) = lat.sin_cos();
    let (sin_lon, cos_lon) = lon.sin_cos();

    // Inverse rotation: ENU -> ECEF delta
    let dx = -sin_lon * enu.x - sin_lat * cos_lon * enu.y + cos_lat * cos_lon * enu.z;
    let dy = cos_lon * enu.x - sin_lat * sin_lon * enu.y + cos_lat * sin_lon * enu.z;
    let dz = cos_lat * enu.y + sin_lat * enu.z;

    let org_ecef = wgs84_to_ecef(origin);
    let target_ecef = org_ecef + DVec3::new(dx, dy, dz);

    ecef_to_wgs84(target_ecef)
}

#[cfg(test)]
mod tests {
    use super::*;

    const MM: f64 = 1e-3; // 1 mm tolerance
    const DEG_TOL: f64 = 1e-9; // ~0.1 mm at equator

    fn approx_eq_dvec3(a: DVec3, b: DVec3, tol: f64) {
        assert!(
            (a.x - b.x).abs() < tol && (a.y - b.y).abs() < tol && (a.z - b.z).abs() < tol,
            "DVec3 mismatch: {a} vs {b} (tol={tol})"
        );
    }

    fn approx_eq_pos(a: &GeoPosition, b: &GeoPosition, deg_tol: f64, alt_tol: f64) {
        assert!(
            (a.lat - b.lat).abs() < deg_tol,
            "lat mismatch: {} vs {} (tol={deg_tol})",
            a.lat,
            b.lat,
        );
        assert!(
            (a.lon - b.lon).abs() < deg_tol,
            "lon mismatch: {} vs {} (tol={deg_tol})",
            a.lon,
            b.lon,
        );
        assert!(
            (a.alt - b.alt).abs() < alt_tol,
            "alt mismatch: {} vs {} (tol={alt_tol})",
            a.alt,
            b.alt,
        );
    }

    #[test]
    fn ecef_round_trip_equator() {
        let pos = GeoPosition::new(0.0, 0.0, 0.0);
        let ecef = wgs84_to_ecef(&pos);
        // At (0,0,0) on ellipsoid, ECEF x == semi-major axis
        assert!((ecef.x - WGS84_A).abs() < MM);
        assert!(ecef.y.abs() < MM);
        assert!(ecef.z.abs() < MM);
        let back = ecef_to_wgs84(ecef);
        approx_eq_pos(&back, &pos, DEG_TOL, MM);
    }

    #[test]
    fn ecef_round_trip_north_pole() {
        let pos = GeoPosition::new(90.0, 0.0, 0.0);
        let ecef = wgs84_to_ecef(&pos);
        assert!(ecef.x.abs() < MM);
        assert!(ecef.y.abs() < MM);
        assert!((ecef.z - WGS84_B).abs() < MM);
        let back = ecef_to_wgs84(ecef);
        approx_eq_pos(&back, &pos, DEG_TOL, MM);
    }

    #[test]
    fn ecef_round_trip_with_altitude() {
        let pos = GeoPosition::new(48.8566, 2.3522, 300.0);
        let ecef = wgs84_to_ecef(&pos);
        let back = ecef_to_wgs84(ecef);
        approx_eq_pos(&back, &pos, DEG_TOL, MM);
    }

    #[test]
    fn ecef_round_trip_southern_hemisphere() {
        let pos = GeoPosition::new(-33.8688, 151.2093, 50.0);
        let ecef = wgs84_to_ecef(&pos);
        let back = ecef_to_wgs84(ecef);
        approx_eq_pos(&back, &pos, DEG_TOL, MM);
    }

    #[test]
    fn enu_round_trip() {
        let origin = GeoPosition::new(48.8566, 2.3522, 0.0);
        let target = GeoPosition::new(48.8570, 2.3530, 10.0);
        let enu = wgs84_to_enu(&target, &origin);
        let back = enu_to_wgs84(enu, &origin);
        approx_eq_pos(&back, &target, DEG_TOL, MM);
    }

    #[test]
    fn enu_origin_is_zero() {
        let origin = GeoPosition::new(40.0, -74.0, 100.0);
        let enu = wgs84_to_enu(&origin, &origin);
        approx_eq_dvec3(enu, DVec3::ZERO, MM);
    }

    #[test]
    fn enu_east_direction() {
        // Moving slightly east should give positive east, near-zero north/up.
        let origin = GeoPosition::new(0.0, 0.0, 0.0);
        let east_point = GeoPosition::new(0.0, 0.001, 0.0);
        let enu = wgs84_to_enu(&east_point, &origin);
        assert!(enu.x > 0.0, "east component should be positive");
        assert!(enu.x.abs() > enu.y.abs() * 100.0);
    }
}
