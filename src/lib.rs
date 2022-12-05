use axum::extract::rejection::JsonRejection;
use axum::response::{IntoResponse, Response};
use axum::Json;
use hyper::StatusCode;
use serde_json::json;
use sqlx::PgPool;

pub mod database;
pub mod endpoints;
pub mod task;
pub mod worker;

#[derive(Debug, Clone)]
pub struct AppState {
    pub db: PgPool,
}

pub struct AppError(Kind);

pub enum Kind {
    Json(String),
    Sqlx(sqlx::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self.0 {
            Kind::Json(error_message) => {
                let err_json = json!({ "error_message": error_message });
                (StatusCode::BAD_REQUEST, Json(err_json)).into_response()
            }
            Kind::Sqlx(error) => {
                // Ideally, I would log an error here with the details and return
                // a non-detailed error to user but there is no need to do that
                // in this assignment's context.
                let err_json = json!({"error_message": error.to_string()});
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_json)).into_response()
            }
        }
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError(Kind::Sqlx(err))
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
    AppError(Kind::Json(message))
}
