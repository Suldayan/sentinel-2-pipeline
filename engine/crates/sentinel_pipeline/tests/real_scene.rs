use std::time::Duration;
use chrono::DateTime;
use sentinel_pipeline::{ingest_pass};
use sentinel_types::SatellitePassEvent;
use sentinel_pipeline::stac::fetch_scene_urls;

const T1: &str = "2015-07-10T00:00:00Z";
const T2: &str = "2026-03-10T23:59:59Z";
const OVERVIEW_LEVEL: u8 = 1;

fn event() -> SatellitePassEvent {
    SatellitePassEvent {
        satellite_id: "SENTINEL-2A".into(),
        pass_start: T1.parse::<DateTime<chrono::Utc>>().unwrap(),
        pass_end: T2.parse::<DateTime<chrono::Utc>>().unwrap(),
        max_elevation_deg: 0.0,

        min_lon: -123.48,
        max_lon: -122.88,
        min_lat:  49.00,
        max_lat:  49.40,


    }
}

/// Verifies the STAC query returns signed URLs for a known Surrey scene.
/// Requires network access — run with `cargo test -- --ignored`.
#[test]
#[ignore]
fn fetch_real_surrey_scene() {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(120))
        .build()
        .unwrap();

    let event = event();
    let result = fetch_scene_urls(
        &client,
        event.min_lon, event.min_lat,
        event.max_lon, event.max_lat,
        &event.pass_start.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        &event.pass_end.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
    );

    assert!(result.is_ok(), "STAC query failed: {:?}", result.err());

    let urls = result.unwrap();
    assert!(urls.is_some(), "No scenes found — try a different date");

    let urls = urls.unwrap();
    println!("B04: {}", urls.b04);
    println!("B08: {}", urls.b08);
}

/// Full end-to-end: STAC → band fetch → NDVI → GeoTIFF written to disk.
/// Requires network access — run with `cargo test -- --ignored`.
#[test]
#[ignore]
fn produces_ndvi_geotiff_for_surrey() {
    let result = ingest_pass(&event(), OVERVIEW_LEVEL);

    assert!(result.is_ok(), "Ingestion failed: {:?}", result.err());

    let path = result.unwrap();
    assert!(path.is_some(), "No imagery available for this date");

    let path = path.unwrap();
    assert!(
        std::path::Path::new(&path).exists(),
        "GeoTIFF not found at {path}"
    );
    println!("Output: {path}");
}

