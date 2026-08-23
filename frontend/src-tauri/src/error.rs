use serde::ser::{Serialize, SerializeStruct, Serializer};
use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("{0}")]
    Unexpected(String),
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
        }
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
}
