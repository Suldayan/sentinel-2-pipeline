//! # sentinel_db
//!
//! Persists NDVI pipeline results to PostgreSQL/PostGIS.
//!
//! Listens for [`sentinel_types::Event::PipelineFinished`] over an `mpsc`
//! channel and inserts summary statistics into the `ndvi_history` table.
//! Retries failed inserts up to 3 times before logging the error and moving on.

mod connection;
mod error;
mod insert;
mod listener;

pub use error::{DbError, DbResult};
pub use insert::insert_ndvi_result;
pub use listener::listen;