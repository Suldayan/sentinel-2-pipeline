use std::thread;
use std::time::Duration;
use log::{info, warn, error};
use sentinel_types::PipelineResult;
use crate::connection::connect;
use crate::error::{DbError, DbResult};

const MAX_ATTEMPTS: u32 = 3;
const RETRY_DELAY: Duration = Duration::from_secs(5);

/// Insert a [`PipelineResult`] into `ndvi_history`.
///
/// Retries up to [`MAX_ATTEMPTS`] times with a [`RETRY_DELAY`] backoff
/// before returning [`DbError::InsertFailed`].
pub fn insert_ndvi_result(result: &PipelineResult) -> DbResult<()> {
    let mut last_err = None;

    for attempt in 1..=MAX_ATTEMPTS {
        match try_insert(result) {
            Ok(()) => {
                info!(
                    "Inserted NDVI record for {} at {}",
                    result.satellite_id, result.captured_at
                );
                return Ok(());
            }
            Err(e) => {
                warn!("Insert attempt {attempt}/{MAX_ATTEMPTS} failed: {e}");
                last_err = Some(e);
                if attempt < MAX_ATTEMPTS {
                    thread::sleep(RETRY_DELAY);
                }
            }
        }
    }

    Err(DbError::InsertFailed {
        attempts: MAX_ATTEMPTS,
        source: last_err.unwrap(),
    })
}

fn try_insert(result: &PipelineResult) -> Result<(), postgres::Error> {
    let mut client = connect().map_err(|e| match e {
        crate::error::DbError::Connection(inner) => inner,
        other => panic!("Unexpected error: {other}"),
    })?;

    client.execute(
        r#"
        INSERT INTO ndvi_history (
            captured_at,
            satellite_id,
            min_lon,
            max_lon,
            min_lat,
            max_lat,
            mean_ndvi,
            max_ndvi,
            min_ndvi,
            valid_pixels,
            tif_path,
            bbox
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11,
            ST_MakeEnvelope($3, $5, $4, $6, 4326)
        )
        "#,
        &[
            &result.captured_at,
            &result.satellite_id,
            &result.min_lon,
            &result.max_lon,
            &result.min_lat,
            &result.max_lat,
            &result.mean_ndvi,
            &result.max_ndvi,
            &result.min_ndvi,
            &(result.valid_pixels as i32),
            &result.tif_path,
        ],
    )?;

    Ok(())
}