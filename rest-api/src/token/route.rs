use axum::{http::StatusCode, routing::get, Json, Router};

pub fn router() -> Router {
    Router::new().route("/health", get(health_check))
}
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthyResponse {
    pub message: String,
}

async fn health_check() -> Result<Json<HealthyResponse>, (StatusCode, String)> {
    Ok(Json(HealthyResponse {
        message: "Healthy".into(),
    }))
}

async fn search_token() -> Result<Json<HealthyResponse>, (StatusCode, String)> {
    Ok(Json(HealthyResponse {
        message: "Healthy".into(),
    }))
}
