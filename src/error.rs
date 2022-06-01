use crate::data_query_lexical::{LexerError, LexicalOperations};
use crate::QueryError::*;

#[derive(Debug)]
pub enum QueryError {
    QueryIsEmpty,
    SerdeError(serde_json::Error),
    LexicalError(LexerError),
    CannotUseIdentifierAsArrayKeyIndex(String),
    UncontrolledError(String),
}

impl From<serde_json::Error> for QueryError {
    fn from(e: serde_json::Error) -> Self {
        SerdeError(e)
    }
}

impl From<LexerError> for QueryError {
    fn from(e: LexerError) -> Self {
        Self::LexicalError(e)
    }
}

impl From<String> for QueryError {
    fn from(s: String) -> Self {
        Self::UncontrolledError(s)
    }
}

impl From<&str> for QueryError {
    fn from(s: &str) -> Self {
        Self::UncontrolledError(s.to_string())
    }
}
