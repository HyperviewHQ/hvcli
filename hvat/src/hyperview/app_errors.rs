use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Input file does not exist")]
    InputFileDoesNotExist,

    #[error("File already exists, can't over write")]
    FileExists,

    #[error("Must provide an output filename")]
    NoOutputFilename,
}
