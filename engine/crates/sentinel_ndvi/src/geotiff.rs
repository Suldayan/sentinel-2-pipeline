use image::RgbImage;
use std::path::Path;
use crate::error::{NdviError, NdviResult};

/// Georeferencing parameters written to the `.tfw` world file.
///
/// All values are in the coordinate system defined by the accompanying `.prj`.
pub struct GeoRef {
    /// Pixel width in map units (positive east).
    pub pixel_size_x: f64,
    /// Pixel height in map units (negative = north-up).
    pub pixel_size_y: f64,
    /// X coordinate of the centre of the top-left pixel.
    pub origin_x: f64,
    /// Y coordinate of the centre of the top-left pixel.
    pub origin_y: f64,
    /// Well-Known Text CRS string written to the `.prj` sidecar.
    pub prj_wkt: &'static str,
}

impl GeoRef {
    /// UTM Zone 10N (EPSG:32610) at 10 m resolution — the native CRS for
    /// Sentinel-2 tiles covering Surrey, BC.
    pub fn utm10n_10m() -> Self {
        Self {
            pixel_size_x:  10.0,
            pixel_size_y: -10.0,
            origin_x: 499_980.0,
            origin_y: 5_500_020.0,
            prj_wkt: r#"PROJCS["WGS 84 / UTM zone 10N",GEOGCS["WGS 84",DATUM["WGS_1984",SPHEROID["WGS 84",6378137,298.257223563]],PRIMEM["Greenwich",0],UNIT["degree",0.0174532925199433]],PROJECTION["Transverse_Mercator"],PARAMETER["latitude_of_origin",0],PARAMETER["central_meridian",-123],PARAMETER["scale_factor",0.9996],PARAMETER["false_easting",500000],PARAMETER["false_northing",0],UNIT["metre",1]]"#,
        }
    }
}

/// Write an RGB-coloured NDVI GeoTIFF plus `.tfw` / `.prj` sidecar files.
///
/// The colour ramp runs:
/// - deep blue  → water / shadow  (NDVI < −0.1)
/// - brown/tan  → bare soil       (NDVI ≈ 0)
/// - yellow     → sparse veg      (NDVI ≈ 0.1)
/// - lime green → moderate veg    (NDVI ≈ 0.4)
/// - dark green → dense canopy    (NDVI → 1.0)
///
/// For full floating-point NDVI output use [`write_f32_tiff`] instead and
/// apply a colour ramp in QGIS.
///
/// # Errors
///
/// Returns [`NdviError`] on buffer size mismatch or I/O failure.
pub fn write_rgb_geotiff(
    ndvi: &[f32],
    width: u32,
    height: u32,
    path: &str,
    georef: &GeoRef,
) -> NdviResult<()> {
    let pixels: Vec<u8> = ndvi.iter()
        .flat_map(|&v| ndvi_to_rgb(v))
        .collect();

    let expected = (width * height * 3) as usize;
    let img = RgbImage::from_raw(width, height, pixels)
        .ok_or(NdviError::BufferMismatch { expected, actual: (width * height) as usize })?;

    img.save_with_format(Path::new(path), image::ImageFormat::Tiff)?;
    write_sidecars(path, georef)?;
    Ok(())
}

/// Write a single-band Float32 TIFF containing raw NDVI values.
///
/// Load this in QGIS with *Singleband pseudocolor* + *RdYlGn* ramp and a
/// 2–98 % percentile stretch for best results.
pub fn write_f32_tiff(
    ndvi:   &[f32],
    width:  u32,
    height: u32,
    path:   &str,
    georef: &GeoRef,
) -> NdviResult<()> {
    use std::fs::File;
    use tiff::encoder::{TiffEncoder, colortype::Gray32Float};

    let file = File::create(path)?;
    let mut enc = TiffEncoder::new(file)
        .map_err(|e| NdviError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;
    enc.write_image::<Gray32Float>(width, height, ndvi)
        .map_err(|e| NdviError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

    write_sidecars(path, georef)?;
    Ok(())
}

fn write_sidecars(tif_path: &str, georef: &GeoRef) -> NdviResult<()> {
    let tfw = format!(
        "{}\n0.0\n0.0\n{}\n{}\n{}\n",
        georef.pixel_size_x, georef.pixel_size_y,
        georef.origin_x, georef.origin_y,
    );
    std::fs::write(tif_path.replace(".tif", ".tfw"), tfw)?;
    std::fs::write(tif_path.replace(".tif", ".prj"), georef.prj_wkt)?;
    Ok(())
}

/// Map an NDVI value in `[−1, 1]` to an RGB triple using a 5-stop colour ramp.
fn ndvi_to_rgb(v: f32) -> [u8; 3] {
    #[rustfmt::skip]
    const RAMP: [(f32, u8, u8, u8); 5] = [
        (-1.0, 0, 0, 128),  // deep blue  – water / shadow
        (-0.1, 80, 60, 10),  // dark brown – bare soil
        ( 0.1, 200, 200, 30),  // yellow     – sparse / dry veg
        ( 0.4, 60, 180, 20),  // lime green – moderate vegetation
        ( 1.0, 0, 80, 0),  // dark green – dense canopy
    ];

    let v = v.clamp(-1.0, 1.0);
    for w in RAMP.windows(2) {
        let (v0, r0, g0, b0) = w[0];
        let (v1, r1, g1, b1) = w[1];
        if v <= v1 {
            let t = (v - v0) / (v1 - v0);
            return [lerp(r0, r1, t), lerp(g0, g1, t), lerp(b0, b1, t)];
        }
    }
    [0, 80, 0]
}

#[inline]
fn lerp(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t).round() as u8
}