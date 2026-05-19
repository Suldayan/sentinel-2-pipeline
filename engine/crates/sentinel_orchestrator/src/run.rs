use chrono::Utc;
use sentinel_types::SatellitePassEvent;
use crate::config::AzureConfig;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let config = AzureConfig::from_env();

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

    match sentinel_pipeline::ingest_pass(&event, config.overview_level) {
        Ok(Some(record)) => {
            log::info!("Pipeline complete — mean NDVI: {:.3}", record.mean_ndvi);
            sentinel_db::insert_ndvi_result(&record)?;
        }
        Ok(None) => log::info!("No imagery available"),
        Err(e)   => return Err(Box::new(e)),
    }

    Ok(())
}