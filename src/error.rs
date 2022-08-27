use std::env::VarError;

#[derive(Debug)]
pub enum AppError {
    Env(VarError),
    SerdeJson(serde_json::Error),
    LineBotSdkError(line_bot_sdk::Error),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Env(errors) => write!(f, "Env var Error: {}", errors),
            AppError::SerdeJson(errors) => write!(f, "serde json error: {}", errors),
            AppError::LineBotSdkError(errors) => write!(f, "line bot sdk error: {}", errors),
        }
    }
}
