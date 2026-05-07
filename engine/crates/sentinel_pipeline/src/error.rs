use thiserror::Error;

#[derive(Debug, Error)]
pub enum PipelineError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("STAC API parse error: {0}")]
    StacParse(String),

    #[error("STAC API timed out after retries")]
    StacTimeout,

    #[error("COG fetch error: {0}")]
    Cog(#[from] sentinel_cog::CogError),

    #[error("NDVI compute error: {0}")]
    Ndvi(#[from] sentinel_ndvi::NdviError),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid bounding box: {0}")]  
    InvalidBBox(String),
}

pub type PipelineResult<T> = Result<T, PipelineError>;