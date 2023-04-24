#[derive(Debug)]
pub struct AppErrors {
    code: StatusCode,
    message:String
}

impl AppErrors {
    pub fn new(code:StatusCode, message: impl Into<String>) -> Self {
        Self { code, message: message.into() }
    }
}

impl IntoResponse for AppErrors {
    fn into_response(self) -> axum::response::Response {
        (
            self.code,
            Json(self.message)
        )
        .into_response()
    }
}