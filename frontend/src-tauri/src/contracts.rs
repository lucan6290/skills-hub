use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HealthCheckResponse {
    pub status: String,
    pub version: String,
}
