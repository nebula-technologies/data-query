use crate::data_query_lexical::LexerError;
use crate::QueryError::*;
use jq_rs::Error;

#[derive(Debug)]
pub enum QueryError {
    QueryIsEmpty,
    SerdeError(serde_json::Error),
    LexicalError(LexerError),
    CannotUseIdentifierAsArrayKeyIndex(String),
    UncontrolledError(String),
    JqError(jq_rs::Error),
}

impl From<jq_rs::Error> for QueryError {
    fn from(e: jq_rs::Error) -> Self {
        JqError(e)
    }
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
