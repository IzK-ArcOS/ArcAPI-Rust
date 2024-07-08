use base64::{DecodeError, Engine};
use base64::engine::general_purpose::STANDARD as B64_STANDARD;
use axum::response::{IntoResponse, Response};
use axum::http::StatusCode;
use std::string::FromUtf8Error;

pub fn from_b64(s: &str) -> Result<String, B64ToStrError> {
    String::from_utf8(B64_STANDARD.decode(s).map_err(B64ToStrError::Base64DecodingError)?).map_err(B64ToStrError::UTF8DecodingError)
}

#[derive(Debug)]
pub enum B64ToStrError {
    Base64DecodingError(DecodeError),
    UTF8DecodingError(FromUtf8Error),
}


impl std::fmt::Display for B64ToStrError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Base64DecodingError(dec_err) => write!(f, "base64 decoding error: {dec_err}"),
            Self::UTF8DecodingError(dec_err) => write!(f, "utf-8 decoding error: {dec_err}"),
        }
    }
}


impl IntoResponse for B64ToStrError {
    fn into_response(self) -> Response {
        (StatusCode::BAD_REQUEST, self.to_string()).into_response()
    }
}
