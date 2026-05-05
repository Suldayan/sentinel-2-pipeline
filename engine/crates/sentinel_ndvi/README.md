# sentinel_ndvi

Pure-compute NDVI processing for Sentinel-2 rasters. Takes raw band data, returns NDVI values and GeoTIFF output — no HTTP, no I/O beyond writing the result.

## Install

```toml
[dependencies]
sentinel_ndvi = "0.1"
```

## Quick start

```rust
use sentinel_ndvi::{compute_ndvi, write_f32_tiff, GeoRef, NdviError};

// b04 and b08 are Raster values from sentinel_cog
let (ndvi, width, height) = compute_ndvi(&b04, &b08)?;

// Write a Float32 GeoTIFF — apply colour ramp in QGIS
write_f32_tiff(&ndvi, width, height, "ndvi.tif", &GeoRef::utm10n_10m())?;
```

## NDVI

NDVI (Normalized Difference Vegetation Index) measures vegetation density using near-infrared and red reflectance:

```
NDVI = (NIR - Red) / (NIR + Red)
```

| NDVI range | Meaning |
|------------|---------|
| < 0 | Water, shadow, cloud |
| ~0 | Bare soil, rock, urban |
| 0.1 – 0.3 | Sparse or dry vegetation |
| 0.3 – 0.6 | Moderate vegetation |
| > 0.6 | Dense, healthy canopy |

## Output formats

**Float32 GeoTIFF** — recommended. Stores raw NDVI values for maximum precision. Load in QGIS with *Singleband pseudocolor* + *RdYlGn* ramp at 2–98% percentile stretch for best results.

```rust
write_f32_tiff(&ndvi, w, h, "ndvi.tif", &GeoRef::utm10n_10m())?;
```

**RGB GeoTIFF** — applies a built-in 5-stop colour ramp (blue → brown → yellow → lime → dark green). Useful for quick previews without QGIS.

```rust
write_rgb_geotiff(&ndvi, w, h, "ndvi.tif", &GeoRef::utm10n_10m())?;
```

## Georeferencing

`GeoRef` controls the `.tfw` world file and `.prj` CRS sidecar written alongside the TIFF:

```rust
// Built-in: UTM Zone 10N, 10 m pixels — covers Surrey/BC Sentinel-2 tiles
let georef = GeoRef::utm10n_10m();

// Custom georeferencing for any tile
let georef = GeoRef {
    pixel_size_x:  10.0,
    pixel_size_y: -10.0,
    origin_x:      499_980.0,
    origin_y:    5_500_020.0,
    prj_wkt: "…",
};
```

## Change detection

```rust
use sentinel_ndvi::{calc_difference_map, DifferenceMap};

let diff: DifferenceMap = calc_difference_map(&past_ndvi, &present_ndvi)?;

println!("Mean change:  {:.3}", diff.mean_change);
println!("Max decline:  {:.3}", diff.max_decline);
println!("Max growth:   {:.3}", diff.max_growth);
```

## Error handling

```rust
match compute_ndvi(&b04, &b08) {
    Err(NdviError::DimensionMismatch { b04, b08 }) => // bands misaligned
    Err(NdviError::LengthMismatch { past, present }) => // diff map inputs differ
    Ok((ndvi, w, h)) => // proceed
}
```

## Used with

[`sentinel_cog`](https://crates.io/crates/sentinel_cog) — fetch Sentinel-2 band rasters via HTTP range requests.

## License

MIT