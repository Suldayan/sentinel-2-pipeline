use bytes::Bytes;
use log::debug;
use crate::error::{CogError, CogResult};
use crate::fetch::fetch_range;
use crate::parse::IfdInfo;

/// Sentinel value written to pixels that have no tile data.
/// `u16::MAX` is used because 0 is a valid reflectance value and would
/// incorrectly map to NDVI 0.0 (bare soil) in downstream compute.
pub const NODATA: u16 = u16::MAX;

#[derive(Debug, Clone)]
pub struct Raster {
    pub pixels: Vec<u16>,
    pub width:  u32,
    pub height: u32,
}

pub fn fetch_tiles(
    client: &reqwest::blocking::Client,
    url: &str,
    tiles: &[(usize, u64, u64)],
) -> CogResult<Vec<(usize, Bytes)>> {
    tiles
        .iter()
        .map(|&(i, offset, len)| {
            debug!("Fetching tile {i}: bytes={offset}-{}", offset + len - 1);
            let bytes = fetch_range(client, url, offset, offset + len - 1)?;
            Ok((i, bytes))
        })
        .collect()
}

/// Decompress and stitch all tiles into a full-sized raster.
/// Delegates to `decode_tiles_region` with the full image extent.
pub fn decode_tiles(tiles: Vec<(usize, Bytes)>, info: &IfdInfo, le: bool) -> CogResult<Raster> {
    let tiles_down = (info.img_h + info.tile_h - 1) / info.tile_h;
    decode_tiles_region(tiles, info, le, 0, 0, info.img_w, info.img_h, info.tiles_across - 1, tiles_down - 1)
}

/// Decompress and stitch a filtered tile subset into a cropped raster.
///
/// `min_col`/`min_row` are the grid coordinates of the top-left tile in the
/// filtered set. Each tile is placed relative to this origin so the output
/// raster covers only the filtered region without surrounding zero padding.
/// Pixels with no tile data are filled with [`NODATA`] (`u16::MAX`).
pub fn decode_tiles_region(
    tiles: Vec<(usize, Bytes)>,
    info: &IfdInfo,
    le: bool,
    min_col: u32,
    min_row: u32,
    out_w: u32,
    out_h: u32,
    max_col: u32,
    max_row: u32,
) -> CogResult<Raster> {
    use flate2::read::ZlibDecoder;
    use std::io::Read;

    let IfdInfo { tile_w, tile_h, tiles_across, .. } = *info;

    let _ = (max_col, max_row); // used by caller to size out_w/out_h

    let mut pixels = vec![NODATA; (out_w * out_h) as usize];

    for (original_index, tile_bytes) in &tiles {
        let mut raw = Vec::new();
        ZlibDecoder::new(tile_bytes.as_ref())
            .read_to_end(&mut raw)
            .map_err(|source| CogError::DecompressFailed { index: *original_index, source })?;

        let tile_pixels: Vec<u16> = raw
            .chunks_exact(2)
            .map(|c| if le { u16::from_le_bytes([c[0], c[1]]) } else { u16::from_be_bytes([c[0], c[1]]) })
            .collect();

        let tile_col = (*original_index as u32) % tiles_across - min_col;
        let tile_row = (*original_index as u32) / tiles_across - min_row;
        let x_start = tile_col * tile_w;
        let y_start = tile_row * tile_h;

        for ty in 0..tile_h {
            let y = y_start + ty;
            if y >= out_h { break; }
            for tx in 0..tile_w {
                let x = x_start + tx;
                if x >= out_w { break; }
                let src = (ty * tile_w + tx) as usize;
                let dst = (y * out_w + x) as usize;
                if src < tile_pixels.len() {
                    pixels[dst] = tile_pixels[src];
                }
            }
        }

        debug!("Decoded tile {original_index} ({tile_col},{tile_row})");
    }

    Ok(Raster { pixels, width: out_w, height: out_h })
}