use testcontainers_modules::{
    postgres::Postgres,
    testcontainers::{Container, runners::SyncRunner, ImageExt},
};

fn start_postgis() -> (Container<Postgres>, String) {
    let container = Postgres::default()
        .with_name("postgis/postgis")
        .with_tag("15-3.3")
        .start()
        .expect("PostGIS failed to start");

    let port = container.get_host_port_ipv4(5432).unwrap();
    let conn_str = format!(
        "host=localhost port={port} user=postgres password=postgres dbname=postgres"
    );

    (container, conn_str)
}

fn setup_schema(client: &mut postgres::Client) {
    client.batch_execute("
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
    ").unwrap();
}

fn set_pipeline_env(conn_str: &str) {
    unsafe {
        std::env::set_var("DATABASE_URL", conn_str);
        std::env::set_var("SATELLITE_ID", "SENTINEL-2A");
        std::env::set_var("MIN_LON", "-122.95");
        std::env::set_var("MAX_LON", "-122.65");
        std::env::set_var("MIN_LAT", "49.05");
        std::env::set_var("MAX_LAT", "49.35");
        std::env::set_var("OVERVIEW_LEVEL", "3");
        std::env::set_var("LOOKBACK_DAYS", "2190");
    }
}

fn assert_ndvi_row(client: &mut postgres::Client) {
    let row = client
        .query_one("SELECT mean_ndvi, valid_pixels FROM ndvi_history", &[])
        .unwrap();

    let mean: f32 = row.get(0);
    let pixels: i32 = row.get(1);

    assert!(mean > -1.0 && mean < 1.0, "NDVI out of range: {mean}");
    assert!(pixels > 0, "Expected valid pixels");

    println!("Mean NDVI: {mean:.3}");
    println!("Valid pixels: {pixels}");
}

#[test]
#[ignore]
fn run_test() {
    let (_container, conn_str) = start_postgis();
    let mut client = postgres::Client::connect(&conn_str, postgres::NoTls).unwrap();

    setup_schema(&mut client);
    set_pipeline_env(&conn_str);

    let result = sentinel_orchestrator::run();
    assert!(result.is_ok(), "Pipeline failed: {:?}", result.err());

    assert_ndvi_row(&mut client);
}