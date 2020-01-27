use lambda_runtime::error::LambdaErrorExt;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum HandlerError {
    #[error("generic error {0:?}")]
    Unknown(anyhow::Error),
}

impl LambdaErrorExt for HandlerError {
    fn error_type(&self) -> &str {
        "HandlerError"
    }
}
