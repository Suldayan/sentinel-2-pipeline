//! # sentinel_cog
//!
//! Stream individual tiles from Sentinel-2 Cloud Optimized GeoTIFFs (COGs)
//! over HTTP range requests — no full-band download required.
//!
//! ## Quick start
//!
//! ```no_run
//! use sentinel_cog::{fetch_overview, fetch_overview_bbox, CogError};
//! use sentinel_types::BBox;
//!
//! let client = reqwest::blocking::Client::new();
//!
//! // All tiles at overview level 3 (~300 m resolution, ~500 KB)
//! let raster = fetch_overview(&client, "https://…/B04.tif", 3)?;
//!
//! // Only tiles intersecting the Lower Mainland at 20 m resolution
//! let bbox = BBox { min_lon: -123.5, max_lon: -122.2, min_lat: 49.0, max_lat: 49.5 };
//! let raster = fetch_overview_bbox(&client, "https://…/B04.tif", 1, &bbox)?;
//! # Ok::<(), CogError>(())
//! ```

mod decode;
mod error;
mod fetch;
mod geo;
pub mod parse;

pub use decode::{Raster, NODATA};
pub use error::{CogError, CogResult};
pub use parse::{IfdInfo, GeoTransform, is_little_endian, parse_subifds, parse_ifd_bytes};

use sentinel_types::BBox;

/// Fetch all tiles at the given overview level.
pub fn fetch_overview(
    client: &reqwest::blocking::Client,
    url: &str,
    overview_level: usize,
) -> CogResult<Raster> {
    let (info, le) = resolve_ifd(client, url, overview_level)?;
    let tile_refs: Vec<(usize, u64, u64)> = info.tile_offsets
        .iter()
        .enumerate()
        .map(|(i, &(off, len))| (i, off, len))
        .collect();
    let tiles = decode::fetch_tiles(client, url, &tile_refs)?;
    decode::decode_tiles(tiles, &info, le)
}

/// Fetch only the tiles intersecting `bbox` at the given overview level.
///
/// Pixels outside fetched tiles are filled with [`NODATA`] (`u16::MAX`) rather
/// than 0, so downstream NDVI compute can distinguish nodata from bare soil.
/// Falls back to fetching all tiles when georeferencing tags are absent.
pub fn fetch_overview_bbox(
    client: &reqwest::blocking::Client,
    url: &str,
    overview_level: usize,
    bbox: &BBox,
) -> CogResult<Raster> {
    let (info, le) = resolve_ifd(client, url, overview_level)?;
    let tile_refs = geo::filter_tiles(&info, bbox);

    if tile_refs.is_empty() {
        return Err(CogError::InvalidHeader("No tiles intersect the given bbox".into()));
    }

    let min_col = tile_refs.iter().map(|(i, _, _)| (*i as u32) % info.tiles_across).min().unwrap();
    let max_col = tile_refs.iter().map(|(i, _, _)| (*i as u32) % info.tiles_across).max().unwrap();
    let min_row = tile_refs.iter().map(|(i, _, _)| (*i as u32) / info.tiles_across).min().unwrap();
    let max_row = tile_refs.iter().map(|(i, _, _)| (*i as u32) / info.tiles_across).max().unwrap();

    let out_w = ((max_col - min_col + 1) * info.tile_w).min(info.img_w);
    let out_h = ((max_row - min_row + 1) * info.tile_h).min(info.img_h);

    let tiles = decode::fetch_tiles(client, url, &tile_refs)?;
    decode::decode_tiles_region(tiles, &info, le, min_col, min_row, out_w, out_h, max_col, max_row)
}

fn resolve_ifd(
    client: &reqwest::blocking::Client,
    url: &str,
    overview_level: usize,
) -> CogResult<(IfdInfo, bool)> {
    let header = fetch::fetch_header(client, url)?;
    let le = is_little_endian(&header)?;
    let subifd_offsets = parse_subifds(&header)?;

    let ifd_offset = subifd_offsets
        .get(overview_level)
        .copied()
        .unwrap_or_else(|| *subifd_offsets.last().unwrap());

    let ifd_bytes = fetch::fetch_ifd_block(client, url, ifd_offset)?;
    let info = parse_ifd_bytes(client, url, &ifd_bytes, le)?;
    Ok((info, le))
}