use satellite_predictor::Observer;
use sentinel_types::BBox;
use dotenvy::dotenv;
use std::env;

#[derive(Debug, Clone)]
pub struct OrchestratorConfig {
    pub norad_id: u32,
    pub satellite_id: String,
    pub observer: Observer,
    pub bbox: BBox,
    pub horizon_hours: f64,
    pub min_elevation_deg: f64,
    pub tle_refresh_hours: f64,
}

pub fn load_config() -> OrchestratorConfig {
    dotenv().ok();

    OrchestratorConfig {
        norad_id: env::var("NORAD_ID").unwrap().parse().unwrap(),
        satellite_id: env::var("SATELLITE_ID").unwrap(),
        observer: Observer::new(
            env::var("OBS_LAT").unwrap().parse().unwrap(),
            env::var("OBS_LON").unwrap().parse().unwrap(),
            env::var("OBS_ALT").unwrap().parse().unwrap(),
        ),
        bbox: BBox::surrey_bc(), 
        horizon_hours: env::var("HORIZON_HOURS").unwrap().parse().unwrap(),
        min_elevation_deg: env::var("MIN_ELEVATION_DEG").unwrap().parse().unwrap(),
        tle_refresh_hours: env::var("TLE_REFRESH_HOURS").unwrap().parse().unwrap(),
    }
}
