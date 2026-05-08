use std::sync::mpsc;
use std::thread;

use dotenvy::dotenv;
use satellite_predictor::Observer;
use sentinel_orchestrator::{OrchestratorConfig, load_config, predict_loop};
use sentinel_pipeline::handle_pass;
use sentinel_events::Event;
use crate::config::load_overview_level;

fn main() {
    env_logger::init();

    let config = load_config();
    let overview_level = load_overview_level();

    let (tx, rx) = mpsc::channel::<Event>();

    let tx_orch = tx.clone();
    thread::spawn(move || predict_loop(tx_orch, config));

    sentinel_pipeline::set_sender(tx.clone());

    for event in rx {
        match event {
            Event::SatellitePass(pass) => {
                let lvl = overview_level;
                thread::spawn(move || handle_pass(pass, overview_level));
            }

            Event::PipelineFinished(result) => {
                println!("Pipeline finished: {:?}", result);
            }
        }
    }
}
