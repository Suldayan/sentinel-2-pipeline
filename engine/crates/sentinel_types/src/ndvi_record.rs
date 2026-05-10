pub struct NdviRecord {
    pub captured_at: chrono::DateTime<chrono::Utc>,
    pub satellite_id: String,
    pub min_lon: f64,
    pub max_lon: f64,
    pub min_lat: f64,
    pub max_lat: f64,
    pub mean_ndvi: f32,
    pub max_ndvi: f32,
    pub min_ndvi: f32,
    pub valid_pixels: usize,
    pub tif_path: String,
}