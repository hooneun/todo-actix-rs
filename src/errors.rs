use actix_web::{error::ResponseError, http::StatusCode, HttpResponse};
use serde::Serialize;
use std::fmt;

#[derive(Debug)]
pub enum AppErrorType {
    DbError,
    NotFoundError,
}

#[derive(Debug)]
pub struct AppError {
    pub message: Option<String>,
    pub cause: Option<String>,
    pub error_type: AppErrorType,
}

impl AppError {
    pub fn message(&self) -> String {
        match &self {
            AppError {
                message: Some(message),
                cause: _,
                error_type: _,
            } => message.clone(),
            AppError {
                message: None,
                cause: _,
                error_type: AppErrorType::NotFoundError,
            } => "The requested item was not found".to_string(),
            _ => "An unexpected error has occurred".to_string(),
        }
    }

    pub fn db_error(error: impl ToString) -> AppError {
        AppError {
            message: None,
            cause: Some(error.to_string()),
            error_type: AppErrorType::DbError,
        }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{:?}", self)
    }
}

#[derive(Serialize)]
pub struct AppErrorResponse {
    pub error: String,
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self.error_type {
            AppErrorType::DbError => StatusCode::INTERNAL_SERVER_ERROR,
            AppErrorType::NotFoundError => StatusCode::NOT_FOUND,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(AppErrorResponse {
            error: self.message(),
        })
    }
}

#[cfg(test)]
mod tests {

    use super::{AppError, AppErrorType};

    #[test]
    fn test_default_message() {
        let db_error = AppError {
            message: None,
            cause: None,
            error_type: AppErrorType::DbError,
        };

        assert_eq!(
            db_error.message(),
            "An unexpected error has occurred".to_string(),
            "Default message should be shown"
        )
    }

    #[test]
    fn test_custom_message() {
        let custom_message = "Unalbe to create item".to_string();
        let db_error = AppError {
            message: Some(custom_message.clone()),
            cause: None,
            error_type: AppErrorType::DbError,
        };

        assert_eq!(
            db_error.message(),
            custom_message,
            "User-facing message should be shown"
        )
    }
}
