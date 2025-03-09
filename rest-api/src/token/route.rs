use axum::{http::StatusCode, routing::get, Json, Router};
use crate::app::AppState;

pub fn router() -> Router<AppState> {
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
