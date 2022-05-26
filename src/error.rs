use crate::data_query_lexer::{LexerError, LexicalOperations};
use crate::QueryError::*;

pub enum QueryError {
    QueryIsEmpty,
    SerdeError(serde_json::Error),
    LexicalError(LexerError),
    CannotUseIdentifierAsArrayKeyIndex(String),
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
