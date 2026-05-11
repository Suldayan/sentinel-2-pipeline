use chrono::{DateTime, TimeZone, Utc};
use satellite_predictor::ScanOptions;
use log::debug;
use crate::config::OrchestratorConfig;
use crate::error::{OrchestratorError, OrchestratorResult};
use sentinel_types::SatellitePassEvent;

fn ms_to_datetime(ms: f64) -> DateTime<Utc> {
    Utc.timestamp_millis_opt(ms as i64)
        .single()
        .expect("timestamp out of range")
}

/// Predict all passes for the next `config.horizon_hours` using an
/// already-fetched TLE, and convert them into [`SatellitePassEvent`]s.
///
/// # Errors
///
/// Returns [`OrchestratorError::Prediction`] if the propagator fails.
pub fn predict_passes(
    tle: &(String, String),
    config: &OrchestratorConfig,
) -> OrchestratorResult<Vec<SatellitePassEvent>> {
    let options = ScanOptions::new(
        Utc::now().timestamp_millis() as f64,
        config.horizon_hours,
        config.min_elevation_deg,
    );

    debug!(
        "Scanning {} hours ahead for NORAD {} (min elev {:.1}°)",
        config.horizon_hours, config.norad_id, config.min_elevation_deg
    );

    let windows = satellite_predictor::passes(&tle.0, &tle.1, &config.observer, &options)
        .map_err(|e| OrchestratorError::Prediction(e.to_string()))?;

    debug!("Found {} pass window(s)", windows.len());

    let events = windows
        .into_iter()
        .map(|w| SatellitePassEvent {
            satellite_id: config.satellite_id.clone(),
            pass_start: ms_to_datetime(w.start_ms),
            pass_end: ms_to_datetime(w.end_ms),
            max_elevation_deg: w.max_elevation_deg,
            min_lon: config.bbox.min_lon,
            max_lon: config.bbox.max_lon,
            min_lat: config.bbox.min_lat,
            max_lat: config.bbox.max_lat,
        })
        .collect();

    Ok(events)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ms_to_datetime_unix_epoch() {
        let dt = ms_to_datetime(0.0);
        assert_eq!(dt.to_rfc3339(), "1970-01-01T00:00:00+00:00");
    }

    #[test]
    fn ms_to_datetime_known_timestamp() {
        let dt = ms_to_datetime(1_775_757_600_000.0);
        assert_eq!(dt.format("%Y-%m-%dT%H:%M:%SZ").to_string(), "2026-04-09T18:00:00Z");
    }

}