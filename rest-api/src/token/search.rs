use axum::{http::StatusCode, Json};

use super::HealthyResponse;

#[allow(dead_code)]
async fn search_token() -> Result<Json<HealthyResponse>, (StatusCode, String)> {
    Ok(Json(HealthyResponse {
        message: "Healthy".into(),
    }))
}
