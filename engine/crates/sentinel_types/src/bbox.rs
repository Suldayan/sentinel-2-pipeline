#[derive(Debug, Clone, Copy)]
pub struct BBox {
    pub min_lon: f64,
    pub max_lon: f64,
    pub min_lat: f64,
    pub max_lat: f64,
}

impl BBox {
    /// Surrey, BC — the default region of interest.
    pub fn surrey_bc() -> Self {
        Self {
            min_lon: -122.95,
            max_lon: -122.65,
            min_lat:   49.05,
            max_lat:   49.35,
        }
    }
}

#[cfg(test)]
mod tests {
    user super::*;
    
    #[test]
    fn surrey_bc_bbox_is_in_correct_quadrant() {
        let bbox = BBox::surrey_bc();
        // Surrey is in the northern hemisphere, western longitude
        assert!(bbox.min_lat > 0.0, "Surrey should be north of equator");
        assert!(bbox.min_lon < 0.0, "Surrey should be west of prime meridian");
        assert!(bbox.max_lat > bbox.min_lat, "max_lat should exceed min_lat");
        assert!(bbox.max_lon > bbox.min_lon, "max_lon should exceed min_lon");
    }
}

