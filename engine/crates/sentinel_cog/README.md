# sentinel_cog

Stream individual tiles from Sentinel-2 Cloud Optimized GeoTIFFs (COGs) via HTTP range requests — no full-band download required.

Sentinel-2 bands are ~200 MB each. This crate fetches only the tiles you need, bringing typical downloads down to ~500 KB at overview resolution or a few MB at full 10 m resolution with a bounding box filter.

## Install

```toml
[dependencies]
sentinel_cog = "0.1"
```

## Quick start

```rust
use sentinel_cog::{fetch_overview, fetch_overview_bbox, CogError};
use sentinel_types::BBox;

let client = reqwest::blocking::Client::new();
let url = "https://…/B04.tif"; // signed Sentinel-2 COG URL

// All tiles at overview level 3 (~300 m resolution, ~500 KB)
let raster = fetch_overview(&client, url, 3)?;
println!("{}x{} pixels", raster.width, raster.height);

// Full 10 m resolution, Surrey BC only
let bbox = BBox::surrey_bc();
let raster = fetch_overview_bbox(&client, url, 0, &bbox)?;
```

## Overview levels

| Level | Resolution | Approx. download per band |
|-------|-----------|--------------------------|
| 0 | 10 m (full) | ~200 MB unfiltered, ~2–5 MB with bbox |
| 1 | 20 m | ~50 MB unfiltered |
| 2 | 60 m | ~6 MB unfiltered |
| 3 | ~300 m | ~500 KB unfiltered |

## How it works

Sentinel-2 COGs store image data in a tiled, multi-resolution structure. Rather than downloading the entire file, `sentinel_cog`:

1. Fetches the first 16 KB to read the TIFF header and locate the Image File Directory
2. Parses IFD tags to find tile byte offsets, image dimensions, and the embedded geotransform (tags 33550 + 33922)
3. When a bounding box is provided, converts it to the TIFF's native CRS (UTM) and filters to only the intersecting tiles
4. Fetches each required tile as a separate HTTP range request
5. Decompresses (Zlib) and stitches tiles into a single contiguous raster

## Output

```rust
pub struct Raster {
    pub pixels: Vec<u16>,  // raw u16 reflectance values
    pub width:  u32,
    pub height: u32,
}
```

Raw `u16` values are returned so callers can compute band math (e.g. NDVI) at full precision without intermediate quantization. See [`sentinel_ndvi`](https://crates.io/crates/sentinel_ndvi) for NDVI computation and GeoTIFF output.

## Error handling

All errors are typed via `CogError`:

```rust
match sentinel_cog::fetch_overview(&client, &url, 3) {
    Err(CogError::Http(e))                        => // retry
    Err(CogError::MissingTag { tag, name })       => // unsupported TIFF
    Err(CogError::DecompressFailed { index, .. }) => // corrupt tile
    Ok(raster)                                    => // use raster
}
```

## Data source

Signed URLs for Sentinel-2 L2A COGs are available free from [Microsoft Planetary Computer](https://planetarycomputer.microsoft.com/).

## License

MIT