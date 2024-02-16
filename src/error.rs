use serde::Serialize;
use std::convert::Infallible;
use thiserror::Error;
use warp::{http::StatusCode, Rejection, Reply};

pub enum Error{
    #[error("Invalid credentials")]
    WrongCredentialError,
    #[error("JWT token creation error")]
    JWTTokenCreationError,
    #[error("JWT token error")]
    JWTTokenError,
    #[error("Unauthorized")]
    Unauthorized,
    #[error("No authorization header")]
    NoAuthHeaderError,
    #[error("Invalid authorization")]
    InvalidAuthError,
    #[error("No permission")]
    NoPermissionError
}

struct ErrorResponse{
    message: String,
    status: String
}

impl warp::reject::Reject for Error{}

pub async fn handle_rejection(err: Rejection) -> std::result::Result<impl Reply, Infallible>{
    let (code, message) = if err.is_not_found(){
        (StatusCode::NOT_FOUND, "Not Found".to_String())
    } else if let Some(e) = err.find::<Error>(){
        match e{
            Error::WrongCredentialError => (StatusCode::FORBIDDEN, e.to_string()),
            Error::JWTTokenCreationError => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string()),
            Error::JWTTokenError => (StatusCode::UNAUTHORIZED, e.to_string()),
            Error::NoPermissionError => (StatusCode::UNAUTHORIZED, e.to_string()),
            _ => (StatusCode::BAD_REQUEST, e.to_string())
        }
    } else if err.find::<warp::reject::MethodNotAllowed>().is_some(){
        (StatusCode::METHOD_NOT_ALLOWED, "Method Not Allowed".to_string())
    } else{
        eprintln!("Unhandled error: {:?}", err);
        (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error".to_string())
    };

    let json = warp::reply::json(&ErrorResponse{
        message,
        status: code.to_string()
    });

    Ok(warp::reply::with_status(json, code))
}