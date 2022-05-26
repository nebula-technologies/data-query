extern crate data_query_lexer;
#[macro_use]
extern crate data_query_proc;
extern crate railsgun;
extern crate regex;
extern crate serde;
extern crate serde_json;

use crate::error::QueryError;
use data_query_lexer::{LexOperator, LexerError, LexicalOperations};
use serde::Serialize;
use serde_json::Value;

mod error;

/// Alias for a `Result` with the error type `serde_json::Error`.
pub type QueryResult<T> = std::result::Result<T, QueryError>;

pub fn query<S: Serialize, Q: TryInto<LexicalOperations>>(s: S, query: Q) -> QueryResult<Value> {
    let lexes = query.try_into().map_err(|e| e.into())?;
    let mut data = serde_json::to_value(s).map_err(|e| e.into())?;
}

pub enum MatchResultExpect {
    Value(Value),
    None,
}

impl From<Value> for MatchResultExpect {
    fn from(m: Value) -> Self {
        Self::Value(m)
    }
}

fn query_processor(data: &Value, query: &mut LexicalOperations) -> Result<Vec<MatchResultExpect>> {
    if query.is_empty() {
        Ok(vec![MatchResultExpect::from(*data)])
    } else {
        let key_query = query.pop().ok_or(QueryError::QueryIsEmpty)?;
        match data {
            Value::Array(v) => {
                match key_query {
                    LexOperator::Identifier(ident) => if let Ok(i) = ident.parse::<usize>(){
                        query_processor(&v[i], query)
                    } else {
                        QueryError::CannotUseIdentifierAsArrayKeyIndex(ident)
                    }
                    LexOperator::Pipe(p) => {NotImplemented!();}
                    LexOperator::Generic(g) => {query_generic_object_index(data, query)}
                }
            }
            Value::Object(m) => {
                let mut tmp_value = Vec::new();
                for (k, v) in m {
                    if key_match_map(&k, &key_query) {
                        tmp_value =
                            vec![tmp_value, query_processor(v, query).unwrap_or_default()].concat();
                    }
                }
                Ok(tmp_value)
            }
            _ => Ok(Vec::new()),
        }
    }
}

pub fn query_generic_object_index() -> Result<Vec<MatchResultExpect>>{

}


fn array_items(v: Value, lex: LexicalOperations) -> {

    let mut tmp_value = Vec::new();
    for (k, v) in v.into_iter().enumerate() {
        if key_match_array(&k, &key_query) {
            tmp_value =
                vec![tmp_value, query_processor(v, query).unwrap_or_default()].concat();
        }
    }
    Ok(tmp_value)
}

#[cfg(test)]
pub mod test {
    use data_query_lexer::LexOperator;

    #[test]
    fn test_proc_macro() {
        let lex: Vec<LexOperator> = precompile_lex!(.metadata[1,2,4-6,hello]);
        println!("{:?}", lex);
    }
}
