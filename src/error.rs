use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("line bot sdk error: {0}")]
    LineBotSdkError(#[from] line_bot_sdk::Error),
}
