use sentinel_cog::Raster;
use crate::error::{NdviError, NdviResult};

pub fn compute_ndvi(b04: &Raster, b08: &Raster) -> NdviResult<(Vec<f32>, u32, u32)> {
    if b04.width != b08.width || b04.height != b08.height {
        return Err(NdviError::DimensionMismatch {
            b04: b04.pixels.len(),
            b08: b08.pixels.len(),
        });
    }
    Ok((compute_ndvi_raw(&b04.pixels, &b08.pixels), b04.width, b04.height))
}

/// Compute NDVI from raw u16 pixel slices.
///
/// Pixels where either band is [`sentinel_cog::NODATA`] (`u16::MAX`) are
/// written as `f32::NAN` — QGIS renders these as transparent, preventing
/// nodata from appearing as bare soil (NDVI 0.0) in the output.
///
/// All other pixels where NIR + Red == 0 are clamped to 0.0.
pub fn compute_ndvi_raw(b04: &[u16], b08: &[u16]) -> Vec<f32> {
    b04.iter()
        .zip(b08.iter())
        .map(|(&red, &nir)| {
            if red == sentinel_cog::NODATA || nir == sentinel_cog::NODATA {
                return f32::NAN;
            }
            let r = red as f32;
            let n = nir as f32;
            let denom = n + r;
            if denom == 0.0 { 0.0 } else { (n - r) / denom }
        })
        .collect()
}