use sentinel_types::BBox;
use crate::parse::IfdInfo;

/// Convert WGS84 lat/lon degrees to UTM Zone 10N easting/northing (metres).
fn latlon_to_utm10n(lat_deg: f64, lon_deg: f64) -> (f64, f64) {
    const A: f64 = 6_378_137.0;
    const F: f64 = 1.0 / 298.257_223_563;
    const E2: f64 = 2.0 * F - F * F;
    const K0: f64 = 0.9996;
    const FALSE_EASTING: f64 = 500_000.0;
    const CENTRAL_MERIDIAN: f64 = -123.0_f64;

    let lat = lat_deg.to_radians();
    let lon = lon_deg.to_radians();
    let lon0 = CENTRAL_MERIDIAN.to_radians();
    let e2 = E2;
    let e4 = e2 * e2;
    let e6 = e4 * e2;

    let n = A / (1.0 - e2 * lat.sin().powi(2)).sqrt();
    let t = lat.tan().powi(2);
    let c = e2 / (1.0 - e2) * lat.cos().powi(2);
    let a_coef = (lon - lon0) * lat.cos();

    let m = A * (
        (1.0 - e2 / 4.0 - 3.0 * e4 / 64.0 - 5.0 * e6 / 256.0) * lat
        - (3.0 * e2 / 8.0 + 3.0 * e4 / 32.0 + 45.0 * e6 / 1024.0) * (2.0 * lat).sin()
        + (15.0 * e4 / 256.0 + 45.0 * e6 / 1024.0) * (4.0 * lat).sin()
        - (35.0 * e6 / 3072.0) * (6.0 * lat).sin()
    );

    let easting = FALSE_EASTING + K0 * n * (
        a_coef
        + (1.0 - t + c) * a_coef.powi(3) / 6.0
        + (5.0 - 18.0 * t + t * t + 72.0 * c - 58.0 * (e2 / (1.0 - e2))) * a_coef.powi(5) / 120.0
    );

    let northing = K0 * (
        m + n * lat.tan() * (
            a_coef.powi(2) / 2.0
            + (5.0 - t + 9.0 * c + 4.0 * c * c) * a_coef.powi(4) / 24.0
            + (61.0 - 58.0 * t + t * t + 600.0 * c - 330.0 * (e2 / (1.0 - e2))) * a_coef.powi(6) / 720.0
        )
    );

    (easting, northing)
}

/// Return `(original_tile_index, offset, byte_count)` for every tile that
/// intersects `bbox`. The original index is preserved so `decode_tiles` can
/// place each tile at the correct position in the output raster.
///
/// Falls back to all tiles when `info.geo` is absent.
pub fn filter_tiles(info: &IfdInfo, bbox: &BBox) -> Vec<(usize, u64, u64)> {
    let geo = match &info.geo {
        Some(g) => g,
        None => return info.tile_offsets
            .iter()
            .enumerate()
            .map(|(i, &(off, len))| (i, off, len))
            .collect(),
    };

    let (utm_min_x, utm_min_y) = latlon_to_utm10n(bbox.min_lat, bbox.min_lon);
    let (utm_max_x, utm_max_y) = latlon_to_utm10n(bbox.max_lat, bbox.max_lon);

    info.tile_offsets
        .iter()
        .enumerate()
        .filter(|(i, _)| {
            let tile_col = (*i as u32) % info.tiles_across;
            let tile_row = (*i as u32) / info.tiles_across;

            let tile_min_x = geo.origin_x + (tile_col * info.tile_w) as f64 * geo.pixel_x;
            let tile_max_x = tile_min_x + info.tile_w as f64 * geo.pixel_x;
            let tile_max_y = geo.origin_y + (tile_row * info.tile_h) as f64 * geo.pixel_y;
            let tile_min_y = tile_max_y + info.tile_h as f64 * geo.pixel_y;

            tile_max_x > utm_min_x
                && tile_min_x < utm_max_x
                && tile_max_y > utm_min_y
                && tile_min_y < utm_max_y
        })
        .map(|(i, &(off, len))| (i, off, len))
        .collect()
}