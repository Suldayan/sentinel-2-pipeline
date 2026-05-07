use log::debug;
use crate::error::{OrchestratorError, OrchestratorResult};

const CELESTRAK_BASE: &str = "https://celestrak.org/NORAD/elements/gp.php";

/// Fetch the two TLE lines for a satellite by NORAD catalog number.
///
/// Returns `(line1, line2)` — the name line is discarded.
///
/// # Errors
///
/// Returns [`OrchestratorError::TleFetch`] on HTTP failure or
/// [`OrchestratorError::TleMalformed`] when the response has fewer than
/// 3 lines.
pub fn fetch_tle(norad_id: u32) -> OrchestratorResult<(String, String)> {
    let url = format!("{CELESTRAK_BASE}?CATNR={norad_id}&FORMAT=TLE");
    debug!("Fetching TLE for NORAD {norad_id} from {url}");

    let body = reqwest::blocking::get(&url)
        .map_err(OrchestratorError::TleFetch)?
        .text()
        .map_err(OrchestratorError::TleFetch)?;

    let lines: Vec<&str> = body.trim().lines().map(str::trim).collect();

    if lines.len() < 3 {
        return Err(OrchestratorError::TleMalformed);
    }

    Ok((lines[1].to_string(), lines[2].to_string()))
}