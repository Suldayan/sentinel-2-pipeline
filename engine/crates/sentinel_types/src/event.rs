use crate::ndvi_record::NdviRecord;

/// A confirmed satellite pass enriched with pipeline metadata.
///
/// This is distinct from [`predictor::PassWindow`], which is the raw
/// propagator output. The orchestrator converts `PassWindow` into this type
/// by attaching the satellite identity and the geographic bbox to monitor.
#[derive(Debug, Clone)]
pub struct SatellitePassEvent {
    /// Human-readable satellite name, e.g. `"SENTINEL-2A"`.
    pub satellite_id: String,
    pub pass_start: chrono::DateTime<chrono::Utc>,
    pub pass_end: chrono::DateTime<chrono::Utc>,
    pub max_elevation_deg: f64,
    /// Bounding box of the region of interest for this pass.
    pub min_lon: f64,
    pub max_lon: f64,
    pub min_lat: f64,
    pub max_lat: f64,
}

#[derive(Debug)]
pub enum Event {
    SatellitePass(SatellitePassEvent),
    PipelineFinished(Result<Option<NdviRecord>, String>),
}
