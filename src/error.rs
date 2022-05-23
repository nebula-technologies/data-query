use crate::Error::*;

pub enum Error {
    QueryIsEmpty,
    SerdeError(serde_json::Error),
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        SerdeError(e)
    }
}
