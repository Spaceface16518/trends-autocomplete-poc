use rocket::{
    http::Status,
    response::Responder,
    tokio::{fs::DirEntry, io},
};
use tantivy::{directory::error::OpenDirectoryError, query};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO error: {0}")]
    IO(#[from] io::Error),
    #[error("tantivy error: {0}")]
    Tantivy(#[from] tantivy::error::TantivyError),
    #[error("error opening directory: {0}")]
    Dir(#[from] OpenDirectoryError),
    #[error("csv error: {0}")]
    CSV(#[from] csv_async::Error),
    #[error("bad data file name \"{0}\"")]
    Name(String),
    #[error("field \"{0}\" not found in schema")]
    Field(&'static str),
    #[error("error parsing query: {0}")]
    Query(#[from] query::QueryParserError),
    #[error("rocket error: {0}")]
    Rocket(#[from] rocket::Error),
}

pub(crate) type Result<T, E = Error> = std::result::Result<T, E>;

/// Convenience function for making an [`Error::Name`] from a directory entry.
pub fn name_error(file: &DirEntry) -> Error {
    // TODO: impl From instead of making free function
    // TODO: use path instead of direntry
    Error::Name(file.file_name().to_string_lossy().to_string())
}

impl<'r, 'o: 'r> Responder<'r, 'o> for Error {
    fn respond_to(self, request: &'r rocket::Request<'_>) -> rocket::response::Result<'o> {
        (Status::InternalServerError, self.to_string()).respond_to(request)
    }
}
