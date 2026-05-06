use sentinel_ndvi::{calc_difference_map, compute_ndvi_raw, NdviError};

#[test]
fn ndvi_of_pure_vegetation_is_near_one() {
    // B04 (red) very low, B08 (NIR) very high = dense healthy vegetation
    let b04 = vec![500u16;  100];
    let b08 = vec![4000u16; 100];
    let ndvi = compute_ndvi_raw(&b04, &b08);
    let mean = ndvi.iter().sum::<f32>() / ndvi.len() as f32;
    assert!(mean > 0.7, "Expected high NDVI for vegetation, got {mean:.3}");
}

#[test]
fn ndvi_of_bare_soil_is_near_zero() {
    let b04 = vec![2000u16; 100];
    let b08 = vec![2200u16; 100];
    let ndvi = compute_ndvi_raw(&b04, &b08);
    let mean = ndvi.iter().sum::<f32>() / ndvi.len() as f32;
    assert!(mean < 0.1, "Expected low NDVI for bare soil, got {mean:.3}");
}

#[test]
fn ndvi_of_water_is_negative() {
    // Water absorbs NIR strongly → negative NDVI
    let b04 = vec![1500u16; 100];
    let b08 = vec![300u16;  100];
    let ndvi = compute_ndvi_raw(&b04, &b08);
    let mean = ndvi.iter().sum::<f32>() / ndvi.len() as f32;
    assert!(mean < 0.0, "Expected negative NDVI for water, got {mean:.3}");
}

#[test]
fn ndvi_returns_zero_when_both_bands_are_zero() {
    // Guard against divide-by-zero on sensor fill values
    let b04 = vec![0u16; 10];
    let b08 = vec![0u16; 10];
    let ndvi = compute_ndvi_raw(&b04, &b08);
    assert!(
        ndvi.iter().all(|&v| v == 0.0),
        "Expected all zeros for zero input, got {ndvi:?}"
    );
}

#[test]
fn ndvi_output_stays_within_valid_range() {
    // All physically meaningful NDVI values are in [-1, 1]
    let b04 = vec![100u16, 500u16, 2000u16, 4000u16, 0u16];
    let b08 = vec![4000u16, 100u16, 2000u16, 0u16,   0u16];
    let ndvi = compute_ndvi_raw(&b04, &b08);
    for v in &ndvi {
        assert!(*v >= -1.0 && *v <= 1.0, "NDVI value {v} is outside [-1, 1]");
    }
}

#[test]
fn ndvi_is_symmetric() {
    // Swapping red and NIR should negate the result
    let b04 = vec![1000u16; 10];
    let b08 = vec![3000u16; 10];
    let forward  = compute_ndvi_raw(&b04, &b08);
    let reversed = compute_ndvi_raw(&b08, &b04);
    for (f, r) in forward.iter().zip(reversed.iter()) {
        assert!((f + r).abs() < 1e-5, "Expected f + r ≈ 0, got {}", f + r);
    }
}

// ── calc_difference_map ──────────────────────────────────────────────────────

#[test]
fn difference_map_detects_vegetation_decline() {
    let past = vec![0.6_f32; 100];
    let present = vec![0.3_f32; 100];
    let diff = calc_difference_map(&past, &present).unwrap();
    assert!(diff.mean_change < 0.0, "Expected negative mean change");
    assert!(diff.max_decline < 0.0, "Expected a decline");
    assert!(diff.max_growth.abs() < 1e-5, "Expected no growth, got {}", diff.max_growth);
}

#[test]
fn difference_map_detects_vegetation_growth() {
    let past = vec![0.3_f32; 100];
    let present = vec![0.7_f32; 100];
    let diff = calc_difference_map(&past, &present).unwrap();
    assert!(diff.mean_change > 0.0, "Expected positive mean change");
    assert!(diff.max_growth  > 0.0, "Expected growth");
    assert!(diff.max_decline.abs() < 1e-5, "Expected no decline, got {}", diff.max_decline);
}

#[test]
fn difference_map_is_zero_for_identical_inputs() {
    let ndvi = vec![0.5_f32; 100];
    let diff = calc_difference_map(&ndvi, &ndvi).unwrap();
    assert!(diff.mean_change.abs() < 1e-5);
    assert!(diff.max_decline.abs() < 1e-5);
    assert!(diff.max_growth.abs() < 1e-5);
}

#[test]
fn difference_map_mean_change_is_mathematically_correct() {
    // All pixels change by exactly +0.2 — easy to verify by hand
    let past = vec![0.4_f32; 10];
    let present = vec![0.6_f32; 10];
    let diff = calc_difference_map(&past, &present).unwrap();
    assert!(
        (diff.mean_change - 0.2).abs() < 1e-5,
        "Expected mean_change ≈ 0.2, got {}", diff.mean_change
    );
}

#[test]
fn difference_map_errors_on_mismatched_lengths() {
    let past = vec![0.5_f32; 100];
    let present = vec![0.5_f32; 50];
    let err = calc_difference_map(&past, &present).unwrap_err();
    assert!(
        matches!(err, NdviError::LengthMismatch { past: 100, present: 50 }),
        "Expected LengthMismatch, got {err}"
    );
}