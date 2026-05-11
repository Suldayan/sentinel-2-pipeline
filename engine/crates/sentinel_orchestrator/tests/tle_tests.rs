use sentinel_orchestrator::OrchestratorError;
use sentinel_types::BBox;

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