#[derive(serde::Serialize)]
pub struct HttpResponse<T: serde::Serialize> {
    pub status_code: i32,
    pub data: T,
}
