use std::time::Duration;
use std::fs;
use chrono::Utc;
use log::{info, error};
use sentinel_ndvi::{compute_ndvi, write_f32_tiff, GeoRef};
use sentinel_types::{SatellitePassEvent, BBox};
use crate::error::{PipelineError, PipelineResult};
use crate::stac::fetch_scene_urls;

/// Fetch bands, compute NDVI, and write a Float32 GeoTIFF.
///
/// Returns the output path on success, or `Ok(None)` when no imagery is
/// available for this pass.
pub fn ingest_pass(event: &SatellitePassEvent, overview_level: u8) -> PipelineResult<Option<String>> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(120))
        .build()
        .map_err(PipelineError::Http)?;

    const FMT: &str = "%Y-%m-%dT%H:%M:%SZ";
    let urls = fetch_scene_urls(
        &client,
        event.min_lon, event.min_lat,
        event.max_lon, event.max_lat,
        &event.pass_start.format(FMT).to_string(),
        &event.pass_end.format(FMT).to_string(),
    )?;

    let Some(urls) = urls else {
        info!("No imagery for pass on {}", event.satellite_id);
        return Ok(None);
    };

    let bbox = build_bbox(event.min_lon, event.max_lon, event.min_lat, event.max_lat)?;

    let b04 = sentinel_cog::fetch_overview_bbox(&client, &urls.b04, overview_level as usize, &bbox)?;
    let b08 = sentinel_cog::fetch_overview_bbox(&client, &urls.b08, overview_level as usize, &bbox)?;

    info!("Bands fetched: {}×{}", b04.width, b04.height);

    let (ndvi, w, h) = compute_ndvi(&b04, &b08)?;

    let (_, tiff_path) = create_ndvi_output_dir()?;

    write_f32_tiff(&ndvi, w, h, &tiff_path, &GeoRef::utm10n_10m())?;
    info!("Saved {tiff_path}");

    Ok(Some(tiff_path))
}

pub fn handle_pass(event: SatellitePassEvent, overview_level: u8) {
    let ready_at = event.pass_end + chrono::Duration::hours(6);
    let wait = (ready_at - Utc::now())
        .to_std()
        .unwrap_or(Duration::ZERO);

    info!(
        "Pass {} ends {}; waiting {:?} before ingestion",
        event.satellite_id, event.pass_end, wait
    );
    std::thread::sleep(wait);

    match ingest_pass(&event, overview_level) {
        Ok(Some(path)) => info!("Ingestion complete: {path}"),
        Ok(None) => info!("No imagery available, skipping"),
        Err(e) => error!("Ingestion failed for {}: {e}", event.satellite_id),
    }
}

fn create_ndvi_output_dir() -> PipelineResult<(String, String)> {
    let timestamp = Utc::now().format("%Y-%m-%dT%H-%M-%SZ").to_string();
    let base = std::env::var("OUTPUT_DIR").unwrap_or_else(|_| "output/ndvi".into());
    let out_dir = format!("{}/{}/", base, timestamp);

    fs::create_dir_all(&out_dir)?;

    let tiff_filename = format!("ndvi_{}.tif", timestamp);
    let tiff_path = format!("{}{}", out_dir, tiff_filename);

    Ok((out_dir, tiff_path))
}

fn build_bbox(
    min_lon: f64, max_lon: f64,
    min_lat: f64, max_lat: f64,
) -> PipelineResult<BBox> {
    if min_lon < -180.0 || max_lon > 180.0 {
        return Err(PipelineError::InvalidBBox(
            format!("longitude out of range: min={min_lon}, max={max_lon}")
        ));
    }
    if min_lat < -90.0 || max_lat > 90.0 {
        return Err(PipelineError::InvalidBBox(
            format!("latitude out of range: min={min_lat}, max={max_lat}")
        ));
    }
    if min_lon >= max_lon {
        return Err(PipelineError::InvalidBBox(
            format!("min_lon ({min_lon}) must be less than max_lon ({max_lon})")
        ));
    }
    if min_lat >= max_lat {
        return Err(PipelineError::InvalidBBox(
            format!("min_lat ({min_lat}) must be less than max_lat ({max_lat})")
        ));
    }

    Ok(BBox { min_lon, max_lon, min_lat, max_lat })
}