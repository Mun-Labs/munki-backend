#[derive(serde::Serialize)]
pub struct HttpResponse<T: serde::Serialize> {
    pub code: i32,
    pub response: T,
    pub last_updated: i64,
}

#[derive(serde::Serialize)]
pub struct HttpPaginationResponse<T: serde::Serialize> {
    pub code: i32,
    pub response: T,
    pub last_updated: i64,
    pub total: i64,
}

#[derive(serde::Serialize)]
pub struct ErrorResponse {
    pub message: String,
    pub response: Option<serde_json::Value>,
}
