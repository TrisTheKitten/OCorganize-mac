use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PathValidationError {
    #[error("Please select a directory.")]
    Empty,
    #[error("Invalid directory path.")]
    InvalidPath,
    #[error("Symlinked directories are not supported.")]
    SymlinkedDirectory,
    #[error("Directory does not exist.")]
    NotFound,
    #[error("Invalid directory: {0}")]
    InvalidDirectory(String),
    #[error("Selected path is not a directory.")]
    NotADirectory,
    #[error("Directory must be readable and writable.")]
    NotAccessible,
}

#[derive(Debug, Error)]
pub enum OrganizeError {
    #[error(transparent)]
    PathValidation(#[from] PathValidationError),
    #[error("Symlinked files are not supported.")]
    SymlinkedFile,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}
