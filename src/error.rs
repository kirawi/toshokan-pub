use axum::{http::StatusCode, response::IntoResponse};

#[derive(Debug)]
pub enum LibraryError {
    /// If the book does not exist
    Missing(String),
}

impl std::fmt::Display for LibraryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LibraryError::Missing(title) => {
                f.write_str(&format!("{} does not exist in the library!", title))
            }
        }
    }
}

impl std::error::Error for LibraryError {}

impl IntoResponse for LibraryError {
    fn into_response(self) -> axum::response::Response {
        (
            StatusCode::NOT_FOUND,
            format!("The requested work was not found"),
        )
            .into_response()
    }
}
