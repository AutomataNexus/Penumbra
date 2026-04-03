//! Distance and interpolation on the WGS84 ellipsoid (spherical approximations).

use crate::position::GeoPosition;

/// Mean Earth radius in meters (for haversine).
const EARTH_RADIUS: f64 = 6_371_000.0;

/// Compute the great-circle distance in meters between two positions using the
/// haversine formula. Altitude is ignored.
pub fn haversine_distance(a: &GeoPosition, b: &GeoPosition) -> f64 {
    let lat1 = a.lat.to_radians();
    let lat2 = b.lat.to_radians();
    let dlat = (b.lat - a.lat).to_radians();
    let dlon = (b.lon - a.lon).to_radians();

    let h = (dlat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (dlon / 2.0).sin().powi(2);
    2.0 * EARTH_RADIUS * h.sqrt().asin()
}

/// Compute the initial bearing (forward azimuth) in degrees from `from` to `to`,
/// measured clockwise from true north (0..360).
pub fn bearing(from: &GeoPosition, to: &GeoPosition) -> f64 {
    let lat1 = from.lat.to_radians();
    let lat2 = to.lat.to_radians();
    let dlon = (to.lon - from.lon).to_radians();

    let y = dlon.sin() * lat2.cos();
    let x = lat1.cos() * lat2.sin() - lat1.sin() * lat2.cos() * dlon.cos();
    let b = y.atan2(x).to_degrees();
    (b + 360.0) % 360.0
}

/// Interpolate along the great circle from `from` to `to` at parameter `t`
/// (0.0 = from, 1.0 = to). Altitude is linearly interpolated.
pub fn great_circle_interpolate(from: &GeoPosition, to: &GeoPosition, t: f64) -> GeoPosition {
    let lat1 = from.lat.to_radians();
    let lon1 = from.lon.to_radians();
    let lat2 = to.lat.to_radians();
    let lon2 = to.lon.to_radians();

    // Angular distance
    let dlat = lat2 - lat1;
    let dlon = lon2 - lon1;
    let a = (dlat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (dlon / 2.0).sin().powi(2);
    let d = 2.0 * a.sqrt().asin();

    if d.abs() < 1e-12 {
        // Points are coincident
        return GeoPosition::new(from.lat, from.lon, from.alt + t * (to.alt - from.alt));
    }

    let sin_d = d.sin();
    let a_coeff = ((1.0 - t) * d).sin() / sin_d;
    let b_coeff = (t * d).sin() / sin_d;

    let x = a_coeff * lat1.cos() * lon1.cos() + b_coeff * lat2.cos() * lon2.cos();
    let y = a_coeff * lat1.cos() * lon1.sin() + b_coeff * lat2.cos() * lon2.sin();
    let z = a_coeff * lat1.sin() + b_coeff * lat2.sin();

    let lat = z.atan2((x * x + y * y).sqrt());
    let lon = y.atan2(x);
    let alt = from.alt + t * (to.alt - from.alt);

    GeoPosition::new(lat.to_degrees(), lon.to_degrees(), alt)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn haversine_known_distance() {
        // Paris -> London ~ 343.5 km (commonly cited)
        let paris = GeoPosition::new(48.8566, 2.3522, 0.0);
        let london = GeoPosition::new(51.5074, -0.1278, 0.0);
        let d = haversine_distance(&paris, &london);
        assert!(
            (d - 343_560.0).abs() < 1_000.0,
            "expected ~343.5 km, got {d}"
        );
    }

    #[test]
    fn haversine_same_point() {
        let p = GeoPosition::new(40.0, -74.0, 0.0);
        assert!(haversine_distance(&p, &p) < 1e-6);
    }

    #[test]
    fn haversine_antipodal() {
        let a = GeoPosition::new(0.0, 0.0, 0.0);
        let b = GeoPosition::new(0.0, 180.0, 0.0);
        let d = haversine_distance(&a, &b);
        let half_circumference = std::f64::consts::PI * 6_371_000.0;
        assert!(
            (d - half_circumference).abs() < 1.0,
            "expected ~{half_circumference}, got {d}"
        );
    }

    #[test]
    fn bearing_north() {
        let a = GeoPosition::new(0.0, 0.0, 0.0);
        let b = GeoPosition::new(1.0, 0.0, 0.0);
        let b_deg = bearing(&a, &b);
        assert!((b_deg - 0.0).abs() < 0.01, "expected ~0, got {b_deg}");
    }

    #[test]
    fn bearing_east() {
        let a = GeoPosition::new(0.0, 0.0, 0.0);
        let b = GeoPosition::new(0.0, 1.0, 0.0);
        let b_deg = bearing(&a, &b);
        assert!((b_deg - 90.0).abs() < 0.01, "expected ~90, got {b_deg}");
    }

    #[test]
    fn great_circle_endpoints() {
        let a = GeoPosition::new(48.8566, 2.3522, 0.0);
        let b = GeoPosition::new(51.5074, -0.1278, 100.0);

        let start = great_circle_interpolate(&a, &b, 0.0);
        assert!((start.lat - a.lat).abs() < 1e-9);
        assert!((start.lon - a.lon).abs() < 1e-9);

        let end = great_circle_interpolate(&a, &b, 1.0);
        assert!((end.lat - b.lat).abs() < 1e-9);
        assert!((end.lon - b.lon).abs() < 1e-9);
        assert!((end.alt - 100.0).abs() < 1e-9);
    }

    #[test]
    fn great_circle_midpoint_distance() {
        let a = GeoPosition::new(0.0, 0.0, 0.0);
        let b = GeoPosition::new(0.0, 90.0, 0.0);
        let mid = great_circle_interpolate(&a, &b, 0.5);

        let d_a = haversine_distance(&a, &mid);
        let d_b = haversine_distance(&mid, &b);
        assert!(
            (d_a - d_b).abs() < 1.0,
            "midpoint should be equidistant: {d_a} vs {d_b}"
        );
    }
}
