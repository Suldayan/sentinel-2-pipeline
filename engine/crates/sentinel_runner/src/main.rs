use std::sync::mpsc;
use std::thread;
use satellite_predictor::Observer;
use sentinel_orchestrator::{OrchestratorConfig, predict_loop};
use sentinel_pipeline::handle_pass;
use sentinel_types::{SatellitePassEvent, BBox};

const OVERVIEW_LEVEL: u8 = 1;

fn main() {
    env_logger::init();

    let config = OrchestratorConfig {
        norad_id: 40697,  
        satellite_id: "SENTINEL-2A".into(),
        observer: Observer::new(49.18, -122.85, 60.0),
        bbox: BBox::surrey_bc(),
        horizon_hours: 24.0,
        min_elevation_deg: 10.0,
        tle_refresh_hours: 12.0,
    };

    let (tx, rx) = mpsc::channel::<SatellitePassEvent>();

    thread::spawn(move || predict_loop(tx, config));

    for event in rx {
        thread::spawn(move || handle_pass(event, OVERVIEW_LEVEL));
    }
}