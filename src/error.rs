#[derive(Debug)]
pub enum LibraryError {
    /// If the [`Book`] already exists in the library
    Exists(String),
    /// If the book does not exist
    Missing(String),
}

impl std::fmt::Display for LibraryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LibraryError::Exists(title) => {
                f.write_str(&format!("{title} is already in the library!"))
            }
            LibraryError::Missing(title) => {
                f.write_str(&format!("{title} does not exist in the library!"))
            }
        }
    }
}

impl std::error::Error for LibraryError {}
