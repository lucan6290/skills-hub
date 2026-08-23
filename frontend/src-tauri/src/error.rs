use serde::ser::{Serialize, SerializeStruct, Serializer};
use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("{0}")]
    Unexpected(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("path error: {0}")]
    PathError(String),

    #[error("database error: {0}")]
    DatabaseError(String),

    #[error("filesystem error: {0}")]
    FileSystemError(String),

    #[error("task error: {0}")]
    TaskError(String),

    #[error("update error: {0}")]
    UpdateError(String),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut response = serializer.serialize_struct("ErrorResponse", 4)?;
        response.serialize_field("ok", &false)?;
        response.serialize_field("code", self.code())?;
        response.serialize_field("message", &self.to_string())?;
        response.serialize_field("detail", &Option::<String>::None)?;
        response.end()
    }
}

impl AppError {
    fn code(&self) -> &'static str {
        match self {
            Self::Unexpected(_) => "INTERNAL_ERROR",
            Self::NotFound(_) => "NOT_FOUND",
            Self::InvalidInput(_) => "INVALID_INPUT",
            Self::PathError(_) => "PATH_ERROR",
            Self::DatabaseError(_) => "DATABASE_ERROR",
            Self::FileSystemError(_) => "FILESYSTEM_ERROR",
            Self::TaskError(_) => "TASK_ERROR",
            Self::UpdateError(_) => "UPDATE_ERROR",
        }
    }
}

impl From<String> for AppError {
    fn from(s: String) -> Self {
        AppError::Unexpected(s)
    }
}

impl From<&str> for AppError {
    fn from(s: &str) -> Self {
        AppError::Unexpected(s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::AppError;

    #[test]
    fn serializes_structured_error_response() {
        let value = serde_json::to_value(AppError::Unexpected("boom".into())).unwrap();

        assert_eq!(value["ok"], false);
        assert_eq!(value["code"], "INTERNAL_ERROR");
        assert_eq!(value["message"], "boom");
        assert!(value["detail"].is_null());
    }

    #[test]
    fn serializes_not_found_error() {
        let value = serde_json::to_value(AppError::NotFound("skill xyz".into())).unwrap();
        assert_eq!(value["code"], "NOT_FOUND");
    }

    #[test]
    fn from_string_conversion() {
        let err: AppError = "test error".into();
        assert!(matches!(err, AppError::Unexpected(_)));
    }
}
