use sentinel_orchestrator::OrchestratorError;
use sentinel_types::BBox;

#[test]
fn surrey_bc_bbox_is_in_correct_quadrant() {
    let bbox = BBox::surrey_bc();
    // Surrey is in the northern hemisphere, western longitude
    assert!(bbox.min_lat > 0.0, "Surrey should be north of equator");
    assert!(bbox.min_lon < 0.0, "Surrey should be west of prime meridian");
    assert!(bbox.max_lat > bbox.min_lat, "max_lat should exceed min_lat");
    assert!(bbox.max_lon > bbox.min_lon, "max_lon should exceed min_lon");
}

#[test]
fn ms_to_datetime_unix_epoch() {
    use sentinel_orchestrator::ms_to_datetime;
    let dt = ms_to_datetime(0.0);
    assert_eq!(dt.to_rfc3339(), "1970-01-01T00:00:00+00:00");
}

#[test]
fn ms_to_datetime_known_timestamp() {
    use sentinel_orchestrator::ms_to_datetime;
    // 2026-04-09T18:00:00Z = 1775757600 seconds = 1775757600000 ms
    let dt = ms_to_datetime(1_775_757_600_000.0);
    assert_eq!(dt.format("%Y-%m-%dT%H:%M:%SZ").to_string(), "2026-04-09T18:00:00Z");
}

/// Verify Celestrak returns a valid 3-line TLE for Sentinel-2A.
/// Run with `cargo test -- --ignored`.
#[test]
#[ignore]
fn fetch_sentinel2a_tle_from_celestrak() {
    use sentinel_orchestrator::fetch_tle;

    let (line1, line2) = fetch_tle(40697).unwrap();

    // TLE line 1 always starts with "1 "
    assert!(line1.starts_with('1'), "TLE line 1 should start with '1', got: {line1}");
    // TLE line 2 always starts with "2 "
    assert!(line2.starts_with('2'), "TLE line 2 should start with '2', got: {line2}");
    // Both lines are exactly 69 characters
    assert_eq!(line1.len(), 69, "TLE line 1 should be 69 chars");
    assert_eq!(line2.len(), 69, "TLE line 2 should be 69 chars");
}

#[test]
#[ignore]
fn invalid_norad_id_returns_malformed_error() {
    use sentinel_orchestrator::fetch_tle;

    // NORAD ID 0 doesn't exist — Celestrak should return empty or error
    let result = fetch_tle(0);
    assert!(
        matches!(result, Err(OrchestratorError::TleMalformed) | Err(OrchestratorError::TleFetch(_))),
        "Expected TleMalformed or TleFetch, got {result:?}"
    );
}