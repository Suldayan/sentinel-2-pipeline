use serde::{Deserialize, Serialize};
use log::{debug, warn};
use crate::error::{PipelineError, PipelineResult};

const SIGN_URL: &str = "https://planetarycomputer.microsoft.com/api/sas/v1/sign";
const STAC_URL: &str = "https://planetarycomputer.microsoft.com/api/stac/v1/search";
const MAX_RETRY: u32  = 3;

#[derive(Debug, Clone, Deserialize)]
pub struct StacResponse {
    pub features: Vec<StacFeature>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StacFeature {
    pub assets: StacAssets,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StacAssets {
    #[serde(rename = "B04")]
    pub b04: StacAsset,
    #[serde(rename = "B08")]
    pub b08: StacAsset,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StacAsset {
    pub href: String,
}

#[derive(Debug, Clone, Deserialize)]
struct SignedResponse {
    pub href: String,
}

pub struct SceneUrls {
    pub b04: String,
    pub b08: String,
}

/// Query the Planetary Computer STAC API for a Sentinel-2 scene covering
/// the given bounding box and time window, then sign the asset URLs.
///
/// Returns `None` when no scene is available (e.g. cloud cover or timing).
pub fn fetch_scene_urls(
    client: &reqwest::blocking::Client,
    min_lon: f64,
    min_lat: f64,
    max_lon: f64,
    max_lat: f64,
    start_time: &str,
    end_time: &str,
) -> PipelineResult<Option<SceneUrls>> {
    let body = serde_json::json!({
        "collections": ["sentinel-2-l2a"],
        "bbox": [min_lon, min_lat, max_lon, max_lat],
        "datetime": format!("{start_time}/{end_time}"),
        "query": { 
            "eo:cloud_cover": { "lt": 80 } 
        },
        "limit": 1,
    });

    debug!("POSTing STAC query for bbox=[{min_lon},{min_lat},{max_lon},{max_lat}]");

    let raw = post_with_retry(client, &body)?;
    let stac: StacResponse = serde_json::from_str(&raw)
        .map_err(|e| PipelineError::StacParse(format!("{e} — body: {}", &raw[..raw.len().min(200)])))?;

    let Some(scene) = stac.features.into_iter().next() else {
        debug!("STAC returned 0 features for this query");
        return Ok(None);
    };

    Ok(Some(SceneUrls {
        b04: sign_url(client, &scene.assets.b04.href)?,
        b08: sign_url(client, &scene.assets.b08.href)?,
    }))
}

fn post_with_retry(
    client: &reqwest::blocking::Client,
    body: &serde_json::Value,
) -> PipelineResult<String> {
    for attempt in 1..=MAX_RETRY {
        let resp = client.post(STAC_URL).json(body).send()
            .map_err(PipelineError::Http)?;

        if resp.status() == 504 {
            warn!("STAC 504 timeout (attempt {attempt}/{MAX_RETRY}), retrying…");
            std::thread::sleep(std::time::Duration::from_secs(5));
            continue;
        }

        return resp.error_for_status()
            .map_err(PipelineError::Http)?
            .text()
            .map_err(PipelineError::Http);
    }
    Err(PipelineError::StacTimeout)
}

fn sign_url(client: &reqwest::blocking::Client, href: &str) -> PipelineResult<String> {
    debug!("Signing URL: {href}");
    let raw = client
        .get(SIGN_URL)
        .query(&[("href", href)])
        .send()
        .map_err(PipelineError::Http)?
        .error_for_status()
        .map_err(PipelineError::Http)?
        .text()
        .map_err(PipelineError::Http)?;

    let signed: SignedResponse = serde_json::from_str(&raw)
        .map_err(|e| PipelineError::StacParse(format!("sign response: {e}")))?;

    Ok(signed.href)
}