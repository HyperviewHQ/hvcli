use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("File already exists, can't over write")]
    FileExists,

    #[error("Must provide an output filename")]
    NoOutputFilename,

    #[error("Must provide a valid ID")]
    InvalidId,

    #[error("Asset not found")]
    AssetNotFound,
}
