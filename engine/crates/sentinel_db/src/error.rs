use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("Database connection failed: {0}")]
    Connection(#[from] postgres::Error),

    #[error("Insert failed after {attempts} attempts: {source}")]
    InsertFailed {
        attempts: u32,
        #[source]
        source: postgres::Error,
    },

    #[error("DATABASE_URL environment variable not set and no default provided")]
    MissingConnectionString,
}

pub type DbResult<T> = Result<T, DbError>;