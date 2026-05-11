use sentinel_cog::Raster;
use crate::error::{NdviError, NdviResult};

#[derive(Debug, Clone)]
pub struct NdviStats {
    pub mean_ndvi: f32,
    pub max_ndvi: f32,
    pub min_ndvi: f32,
    pub valid_pixels: usize,
}

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

/// Compute summary statistics from an NDVI slice, skipping NAN pixels.
///
/// Returns `None` if there are no valid pixels at all.
pub fn compute_stats(ndvi: &[f32]) -> Option<NdviStats> {
    let valid: Vec<f32> = ndvi.iter().copied().filter(|v| !v.is_nan()).collect();

    if valid.is_empty() {
        return None;
    }

    let mean = valid.iter().sum::<f32>() / valid.len() as f32;
    let max = valid.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let min = valid.iter().cloned().fold(f32::INFINITY, f32::min);

    Some(NdviStats {
        mean_ndvi: mean,
        max_ndvi: max,
        min_ndvi: min,
        valid_pixels: valid.len(),
    })
}

#[cfg(test)]
mod tests {

    #[test]
    fn stats_returns_none_for_all_nan() {
        let ndvi = vec![f32::NAN; 100];
        assert!(compute_stats(&ndvi).is_none());
    }

    #[test]
    fn stats_skips_nan_pixels() {
        let mut ndvi = vec![0.5_f32; 100];
        ndvi[0] = f32::NAN;
        ndvi[1] = f32::NAN;

        let stats = compute_stats(&ndvi).unwrap();
        assert_eq!(stats.valid_pixels, 98);
        assert!((stats.mean_ndvi - 0.5).abs() < 1e-5);
    }

    #[test]
    fn stats_mean_is_correct() {
        let ndvi = vec![0.2_f32, 0.4, 0.6, 0.8];
        let stats = compute_stats(&ndvi).unwrap();
        assert!((stats.mean_ndvi - 0.5).abs() < 1e-5);
    }

    #[test]
    fn stats_max_and_min_are_correct() {
        let ndvi = vec![0.1_f32, 0.5, 0.9, -0.3];
        let stats = compute_stats(&ndvi).unwrap();
        assert!((stats.max_ndvi - 0.9).abs() < 1e-5);
        assert!((stats.min_ndvi - (-0.3)).abs() < 1e-5);
    }

    #[test]
    fn stats_single_valid_pixel() {
        let mut ndvi = vec![f32::NAN; 99];
        ndvi.push(0.7);
        let stats = compute_stats(&ndvi).unwrap();
        assert_eq!(stats.valid_pixels, 1);
        assert!((stats.mean_ndvi - 0.7).abs() < 1e-5);
    }

    #[test]
    fn stats_all_valid_pixels_counted() {
        let ndvi = vec![0.4_f32; 1000];
        let stats = compute_stats(&ndvi).unwrap();
        assert_eq!(stats.valid_pixels, 1000);
    }
}