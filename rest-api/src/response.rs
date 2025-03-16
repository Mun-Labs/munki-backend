#[derive(serde::Serialize)]
pub struct HttpResponse<T: serde::Serialize> {
    pub code: i32,
    pub response: T,
}
