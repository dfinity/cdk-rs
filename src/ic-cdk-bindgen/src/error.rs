
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IcCdkBindgenError {
    #[error("Custom error: {0}")]
    Custom(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}