use std::sync::mpsc;
use std::thread;
use sentinel_orchestrator::{load_config, predict_loop};
use sentinel_pipeline::pass::handle_pass;
use sentinel_types::{SatellitePassEvent, Event};

fn main() {
    env_logger::init();

    let config = load_config();
    let overview_level = config.overview_level;

    let (pass_tx, pass_rx) = mpsc::channel::<SatellitePassEvent>();

    let (event_tx, event_rx) = mpsc::channel::<Event>();

    sentinel_db::listen(event_rx);

    thread::spawn(move || predict_loop(pass_tx, config));

    for pass in pass_rx {
        let tx = event_tx.clone();
        thread::spawn(move || handle_pass(tx, pass, overview_level));
    }
}