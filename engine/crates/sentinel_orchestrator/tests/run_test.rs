use testcontainers_modules::{
    postgres::Postgres,
    testcontainers::{Container, runners::SyncRunner, ImageExt},
};
use sentinel_orchestrator::Config;

fn start_postgis() -> (Container<Postgres>, String) {
    let container = Postgres::default()
        .with_name("postgis/postgis")
        .with_tag("15-3.3")
        .start()
        .expect("PostGIS container failed to start — is Docker running?");

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
    ").expect("Schema migration failed");
}

fn run_pipeline_at_level(overview_level: u8) -> (f32, i32) {
    let (_container, conn_str) = start_postgis();
    let mut client = postgres::Client::connect(&conn_str, postgres::NoTls).unwrap();
    setup_schema(&mut client);

    sentinel_orchestrator::run_with(Config::for_test(overview_level, conn_str.clone()))
        .unwrap_or_else(|e| panic!("Pipeline failed at level {overview_level}: {e}"));

    let row = client
        .query_one("SELECT mean_ndvi, valid_pixels FROM ndvi_history", &[])
        .unwrap();

    (row.get(0), row.get(1))
}

/// Full resolution — main IFD, largest pixel count.
#[test]
#[ignore]
fn pipeline_overview_level_0() {
    let (mean, pixels) = run_pipeline_at_level(0);
    assert!(mean > -1.0 && mean < 1.0, "NDVI out of range: {mean}");
    assert!(pixels > 0, "Expected valid pixels at level 0");
    println!("Level 0 — mean NDVI: {mean:.3}, valid pixels: {pixels}");
}

/// First overview — coarser resolution, fewer pixels than level 0.
#[test]
#[ignore]
fn pipeline_overview_level_1() {
    let (mean, pixels) = run_pipeline_at_level(1);
    assert!(mean > -1.0 && mean < 1.0, "NDVI out of range: {mean}");
    assert!(pixels > 0, "Expected valid pixels at level 1");
    println!("Level 1 — mean NDVI: {mean:.3}, valid pixels: {pixels}");
}

/// Requesting a level beyond what the COG provides should fail explicitly
/// rather than silently returning the wrong data.
#[test]
#[ignore]
fn pipeline_overview_level_out_of_range() {
    let result = sentinel_orchestrator::run_with(Config::for_test(2, "unused".into()));
    assert!(result.is_err(), "Expected error for unavailable overview level");
}

/// Proves that level 0 and level 1 resolve to distinct IFDs by comparing
/// pixel counts — coarser overviews must cover fewer pixels for the same bbox.
#[test]
#[ignore]
fn overview_levels_resolve_distinct_ifds() {
    let (_, pixels_0) = run_pipeline_at_level(0);
    let (_, pixels_1) = run_pipeline_at_level(1);

    assert!(
        pixels_0 > pixels_1,
        "Level 0 should have more pixels than level 1: {} vs {}",
        pixels_0, pixels_1,
    );
}