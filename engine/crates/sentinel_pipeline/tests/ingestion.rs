use std::time::Duration;
use chrono::DateTime;
use sentinel_pipeline::ingest_pass;
use sentinel_pipeline::stac::fetch_scene_urls;
use sentinel_types::SatellitePassEvent;

// Known good Sentinel-2A acquisition over Surrey, BC — tile T10UEV, 2024-07-15.
// Confirmed present in the STAC catalogue at time of writing.
fn surrey_event() -> SatellitePassEvent {
    SatellitePassEvent {
        satellite_id: "SENTINEL-2A".into(),
        pass_start: "2020-07-10T00:00:00Z".parse::<DateTime<chrono::Utc>>().unwrap(),
        pass_end: "2026-03-10T23:59:59Z".parse::<DateTime<chrono::Utc>>().unwrap(),
        max_elevation_deg: 0.0,
        min_lon: -122.95,
        max_lon: -122.65,
        min_lat: 49.05,
        max_lat: 49.35,
    }
}

fn blocking_client() -> reqwest::blocking::Client {
    reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(120))
        .build()
        .unwrap()
}

/// Verifies the STAC query returns signed URLs for a known Surrey scene.
/// Requires network access — run with `cargo test -- --include-ignored`.
#[test]
#[ignore]
fn stac_returns_urls_for_known_surrey_scene() {
    let client = blocking_client();
    let event = surrey_event();

    let urls = fetch_scene_urls(
        &client,
        event.min_lon, event.min_lat,
        event.max_lon, event.max_lat,
        &event.pass_start.to_rfc3339(),
        &event.pass_end.to_rfc3339(),
    )
    .expect("STAC query failed")
    .expect("No scene found for known-good date — catalogue may have changed");

    assert!(urls.b04.starts_with("https://"), "B04 URL looks wrong: {}", urls.b04);
    assert!(urls.b08.starts_with("https://"), "B08 URL looks wrong: {}", urls.b08);

    println!("B04: {}", urls.b04);
    println!("B08: {}", urls.b08);
}

/// Full end-to-end: STAC → band fetch → NDVI → GeoTIFF written to disk.
/// Requires network access — run with `cargo test -- --include-ignored`.
#[test]
#[ignore]
fn ingest_pass_produces_ndvi_geotiff_for_surrey() {
    let record = ingest_pass(&surrey_event(), 3)
        .expect("Ingestion failed")
        .expect("No imagery available for known-good date");

    assert!(
        std::path::Path::new(&record.tif_path).exists(),
        "GeoTIFF not found at {:?}", record.tif_path
    );
    println!("Output: {}", record.tif_path);
    println!("Mean NDVI: {:.3}", record.mean_ndvi);
    println!("Valid pixels: {}", record.valid_pixels);
}