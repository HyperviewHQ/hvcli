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

    #[error(
        "Invalid date range arguments. Provide either --year and --month, or --start and --end (both)."
    )]
    InvalidDateRangeArgs,

    #[error("Invalid date format: {0}. Expected YYYY-MM-DD.")]
    InvalidDateFormat(String),

    #[error(
        "Bulk operation completed with {failed} failure(s) out of {total} row(s); see log for details."
    )]
    BulkOperationFailures { failed: usize, total: usize },
}
