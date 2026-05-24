pub struct Config {
    pub database_url: String,
    pub satellite_id: String,
    pub min_lon: f64,
    pub max_lon: f64,
    pub min_lat: f64,
    pub max_lat: f64,
    pub overview_level: u8,
    pub lookback_days: i64,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            database_url: std::env::var("DATABASE_URL")
                .expect("DATABASE_URL must be set"),
            satellite_id: std::env::var("SATELLITE_ID")
                .unwrap_or_else(|_| "SENTINEL-2A".into()),
            min_lon: std::env::var("MIN_LON")
                .unwrap_or_else(|_| "-122.95".into())
                .parse()
                .expect("MIN_LON must be a valid f64"),
            max_lon: std::env::var("MAX_LON")
                .unwrap_or_else(|_| "-122.65".into())
                .parse()
                .expect("MAX_LON must be a valid f64"),
            min_lat: std::env::var("MIN_LAT")
                .unwrap_or_else(|_| "49.05".into())
                .parse()
                .expect("MIN_LAT must be a valid f64"),
            max_lat: std::env::var("MAX_LAT")
                .unwrap_or_else(|_| "49.35".into())
                .parse()
                .expect("MAX_LAT must be a valid f64"),
            overview_level: std::env::var("OVERVIEW_LEVEL")
                .unwrap_or_else(|_| "1".into())
                .parse()
                .expect("OVERVIEW_LEVEL must be a valid u8"),
            lookback_days: std::env::var("LOOKBACK_DAYS")
                .unwrap_or_else(|_| "5".into())
                .parse()
                .expect("LOOKBACK_DAYS must be a valid i64"),
        }
    }

    pub fn for_test(overview_level: u8, database_url: String) -> Self {
        Self {
            database_url,
            satellite_id: "SENTINEL-2".into(),
            min_lon: -123.95,
            max_lon: -122.65,
            min_lat: 49.05,
            max_lat: 49.35,
            overview_level,
            lookback_days: 400,
        }
    }
}