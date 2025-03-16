use axum::{http::StatusCode, Json};

use super::HealthyResponse;

#[allow(dead_code)]
pub async fn health() -> Result<Json<HealthyResponse>, (StatusCode, String)> {
    Ok(Json(HealthyResponse {
        message: "Healthy".into(),
    }))
}
