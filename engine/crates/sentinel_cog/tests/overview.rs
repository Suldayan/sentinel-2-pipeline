use sentinel_cog::{CogError};

/// Fetch a real Sentinel-2 overview and verify its dimensions are plausible.
/// Run with `cargo test -- --ignored`.
#[test]
#[ignore]
fn fetch_real_sentinel2_overview() {
    // Sentinel-2A B04 band — Surrey, BC tile T10UEV
    // This URL may expire; replace with a fresh signed URL if needed.
    let url = std::env::var("SENTINEL_B04_URL")
        .expect("Set SENTINEL_B04_URL to a signed Sentinel-2 COG URL");

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .unwrap();

    let raster = sentinel_cog::fetch_overview(&client, &url, 3).unwrap();

    // Level 3 overview of a 10980x10980 tile should be roughly 1372x1372
    assert!(raster.width > 100, "Width {} seems too small", raster.width);
    assert!(raster.height > 100, "Height {} seems too small", raster.height);
    assert_eq!(
        raster.pixels.len(),
        (raster.width * raster.height) as usize,
        "Pixel buffer length doesn't match dimensions"
    );

    println!("Raster: {}x{} ({} pixels)", raster.width, raster.height, raster.pixels.len());
}