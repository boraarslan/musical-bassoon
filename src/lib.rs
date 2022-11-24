use axum::{
    extract::rejection::JsonRejection,
    response::{IntoResponse, Response},
    Json,
};
use hyper::StatusCode;
use serde_json::json;

pub mod endpoints;
pub mod redis;
pub mod task;
pub mod worker;

pub struct State {
    pub db: bb8::Pool<bb8_redis::RedisConnectionManager>,
}

pub struct AppError(Kind);

pub enum Kind {
    Anyhow(anyhow::Error),
    Json(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self.0 {
            Kind::Anyhow(error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error_message": error.to_string()})),
            )
                .into_response(),
            Kind::Json(error_message) => (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error_message": error_message })),
            )
                .into_response(),
        }
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(Kind::Anyhow(err.into()))
    }
}

fn handle_json_error(error: JsonRejection) -> AppError {
    let message = match error {
        JsonRejection::JsonDataError(_) => {
            "Couldn't deserialize the body into the target type".to_string()
        }
        JsonRejection::JsonSyntaxError(_) => "Syntax error in the body".to_string(),
        JsonRejection::MissingJsonContentType(_) => {
            "Request didn't have `Content-Type: application/json` header".to_string()
        }
        JsonRejection::BytesRejection(_) => "Failed to extract the request body".to_string(),
        _ => "Unknown error".to_string(),
    };
    AppError(crate::Kind::Json(message))
}
