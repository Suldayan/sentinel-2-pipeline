CREATE EXTENSION IF NOT EXISTS postgis;

CREATE TABLE ndvi_history (
    id SERIAL PRIMARY KEY,
    captured_at TIMESTAMPTZ NOT NULL,
    satellite_id TEXT NOT NULL,
    min_lon DOUBLE PRECISION NOT NULL,
    max_lon DOUBLE PRECISION NOT NULL,
    min_lat DOUBLE PRECISION NOT NULL,
    max_lat DOUBLE PRECISION NOT NULL,
    mean_ndvi REAL NOT NULL,
    max_ndvi REAL NOT NULL,
    min_ndvi REAL NOT NULL,
    valid_pixels INTEGER NOT NULL,
    tif_path TEXT NOT NULL,
    bbox GEOMETRY(POLYGON, 4326)
);

CREATE INDEX ndvi_history_bbox_idx ON ndvi_history USING GIST (bbox);

CREATE INDEX ndvi_history_time_idx ON ndvi_history (captured_at);