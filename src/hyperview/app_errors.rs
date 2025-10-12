use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("File already exists, can't over write")]
    FileExists,

    #[error("Must provide an output filename")]
    NoOutputFilename,

    #[error("Asset not found")]
    AssetNotFound,

    #[error("Unable to continue with operation; Multiple values detected for property")]
    MultipleValuesDetectedForProperty,

    #[error("Unable to continue with operation; Asset does not have a property named {0}")]
    AssetDoesNotHavePropertyName(String),
}
