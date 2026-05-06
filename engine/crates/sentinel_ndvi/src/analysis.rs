use serde::Serialize;
use crate::error::{NdviError, NdviResult};

#[derive(Debug, Clone, Serialize)]
pub struct DifferenceMap {
    pub mean_change: f32,
    pub max_decline: f32,
    pub max_growth: f32,
    /// Number of valid (non-NAN) pixels used in the calculation.
    pub valid_pixels: usize,
}

/// Compute a [`DifferenceMap`] between two temporally aligned NDVI grids.
///
/// NAN pixels (nodata) are skipped in all calculations. If no valid pixels
/// exist after filtering, returns [`NdviError::LengthMismatch`].
pub fn calc_difference_map(past: &[f32], present: &[f32]) -> NdviResult<DifferenceMap> {
    if past.len() != present.len() {
        return Err(NdviError::LengthMismatch {
            past: past.len(),
            present: present.len(),
        });
    }

    let mut total = 0.0_f32;
    let mut decline = 0.0_f32;
    let mut growth = 0.0_f32;
    let mut count = 0usize;

    for (p, q) in past.iter().zip(present.iter()) {
        if p.is_nan() || q.is_nan() { continue; }
        let diff = q - p;
        total += diff;
        if diff < decline { decline = diff; }
        if diff > growth { growth  = diff; }
        count += 1;
    }

    if count == 0 {
        return Err(NdviError::LengthMismatch { past: 0, present: 0 });
    }

    Ok(DifferenceMap {
        mean_change: total / count as f32,
        max_decline: decline,
        max_growth: growth,
        valid_pixels: count,
    })
}