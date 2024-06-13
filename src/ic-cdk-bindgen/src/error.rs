use thiserror::Error;

#[derive(Error, Debug)]
pub enum IcCdkBindgenError {
    #[error("Custom error: {0}")]
    Custom(String),

    #[error("Io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Principal error: {0}")]
    Principal(#[from] candid::types::principal::PrincipalError),

    #[error("Environment variable not found: {var}")]
    EnvVarNotFound {
        var: String,
        source: std::env::VarError,
    },
}
