use chrono::Utc;
use sentinel_types::SatellitePassEvent;
use crate::config::Config;

/// Entry point for production — reads config from environment.
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    run_with(Config::from_env())
}

/// Runs the pipeline with an explicit config — used directly in tests
/// to avoid env var mutation.
pub fn run_with(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    let event = SatellitePassEvent {
        satellite_id: config.satellite_id,
        pass_start: Utc::now() - chrono::Duration::days(config.lookback_days),
        pass_end: Utc::now(),
        max_elevation_deg: 0.0,
        min_lon: config.min_lon,
        max_lon: config.max_lon,
        min_lat: config.min_lat,
        max_lat: config.max_lat,
    };

    match sentinel_pipeline::ingest_pass(&event, config.overview_level)? {
        Some(record) => {
            log::info!("Pipeline complete — mean NDVI: {:.3}", record.mean_ndvi);
            sentinel_db::insert_ndvi_result(&record, &config.database_url)?;
        }
        None => log::info!("No imagery available for this pass"),
    }

    Ok(())
}