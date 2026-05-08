use std::sync::mpsc::Receiver;
use std::thread;
use log::{info, warn, error};
use sentinel_types::Event;
use crate::insert::insert_ndvi_result;

/// Spawn a background thread that listens for [`Event`]s and writes
/// [`Event::PipelineFinished`] results to the database.
///
/// Returns when the sender is dropped (channel closed).
pub fn listen(rx: Receiver<Event>) {
    thread::spawn(move || {
        info!("DB listener started");

        for event in rx {
            match event {
                Event::PipelineFinished(Ok(Some(result))) => {
                    if let Err(e) = insert_ndvi_result(&result) {
                        error!("Failed to insert NDVI result after retries: {e}");
                    }
                }
                Event::PipelineFinished(Ok(None)) => {
                    info!("No imagery available for this pass — skipping DB insert");
                }
                Event::PipelineFinished(Err(e)) => {
                    warn!("Pipeline reported failure: {e}");
                }
                Event::SatellitePass(_) => {}
            }
        }

        info!("DB listener shutting down — channel closed");
    });
}