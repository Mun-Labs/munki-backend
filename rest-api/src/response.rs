use chrono::Utc;

#[derive(serde::Serialize)]
pub struct HttpResponse<T: serde::Serialize> {
    pub code: i32,
    pub response: T,
    #[serde(default = "default_last_updated")]
    pub last_updated: i64,
}

fn default_last_updated() -> i64 {
    Utc::now().timestamp()
}
