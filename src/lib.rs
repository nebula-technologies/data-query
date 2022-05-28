extern crate data_query_lexer;
#[macro_use]
extern crate data_query_proc;
extern crate railsgun;
extern crate regex;
extern crate serde;
extern crate serde_json;

use crate::error::QueryError;
use data_query_lexer::{GenericObjectIndex, LexOperator, LexerError, LexicalOperations, Slicer};
use serde::Serialize;
use serde_json::Value;

mod error;

/// Alias for a `Result` with the error type `serde_json::Error`.
pub type QueryResult<T> = std::result::Result<T, QueryError>;

pub fn query<S: Serialize, Q: TryInto<LexicalOperations>>(
    s: S,
    query: Q,
) -> QueryResult<Vec<Value>> {
    let mut lexes = query.try_into().map_err(|e| e.into())?;
    let mut data = serde_json::to_value(s).map_err(|e| e.into())?;
    let mut results = Vec::new();
    query_processor(&data, &mut lexes, &mut results)?;
    Ok(results)
}

fn query_processor(
    data: &Value,
    query: &mut LexicalOperations,
    results: &mut Vec<Value>,
) -> QueryResult<()> {
    if query.is_empty() {
        results.push(MatchResultExpect::from(data.clone()));
        Ok(())
    } else {
        let key_query = query.pop().ok_or(QueryError::QueryIsEmpty)?;
        match data {
            Value::Array(v) => match key_query {
                LexOperator::Identifier(ident) => {
                    if let Ok(i) = ident.parse::<usize>() {
                        query_processor(&v[i], query, results)
                    } else {
                        QueryError::CannotUseIdentifierAsArrayKeyIndex(ident)
                    }
                }
                LexOperator::Pipe(p) => {
                    NotImplemented!();
                }
                LexOperator::Generic(mut g) => {
                    query_generic_object_index(v, &mut g, query, results)
                }
            },
            Value::Object(m) => {
                let mut tmp_value = Vec::new();
                for (k, v) in m {
                    if key_match_map(&k, &key_query) {
                        query_processor(v, query, results)
                    }
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }
    Ok(())
}

pub fn query_generic_object_index(
    data: &Vec<Value>,
    index_match: &mut GenericObjectIndex,
    query: &mut LexicalOperations,
    results: &mut Vec<Value>,
) -> QueryResult<()> {
    let key_query = index_match.pop().ok_or(QueryError::QueryIsEmpty)?;
    for (k, v) in data.iter().enumerate() {
        if match_slice_to_key(k, index_match) {
            query_processor(value, query, results)?
        }
    }
    Ok(())
}

pub fn match_slice_to_key(key: usize, query: &mut GenericObjectIndex) -> bool {
    match query {
        GenericObjectIndex::Wildcard => true,
        GenericObjectIndex::Slice(slice) => {
            for s in slice {
                match s {
                    Slicer::Index(i) => {
                        if key == i {
                            true
                        }
                    }
                    Slicer::Slice(f, t) => {
                        if key >= *f && key <= *t {
                            true
                        }
                    }
                    Slicer::Ident(ident) => {
                        if Ok(i) = ident.parse::<usize>() {
                            if key == i {
                                true
                            }
                        }
                    }
                }
            }
            return false;
        }
    }
}

// fn array_items(v: Value, lex: LexicalOperations) -> {
//
//     let mut tmp_value = Vec::new();
//     for (k, v) in v.into_iter().enumerate() {
//         if key_match_array(&k, &key_query) {
//             tmp_value =
//                 vec![tmp_value, query_processor(v, query).unwrap_or_default()].concat();
//         }
//     }
//     Ok(tmp_value)
// }

#[cfg(test)]
pub mod test {
    use crate::query;
    use data_query_lexer::LexOperator;

    #[derive(Serialize)]
    pub struct User {
        id: String,
        is_active: bool,
        balance: String,
        age: i32,
        eye_color: String,
        name: String,
        gender: String,
        company: String,
        email: String,
        phone: String,
        friends: Vec<Friend>,
        favorite_fruit: String,
    }

    impl Default for User {
        fn default() -> Self {
            Self {
                id: "5973782bdb9a930533b05cb2".into(),
                is_active: true,
                balance: "$1,446.35".into(),
                age: 32,
                eye_color: "green".into(),
                name: "Logan Keller".into(),
                gender: "male".into(),
                company: "ARTIQ".into(),
                email: "logankeller@artiq.com".into(),
                phone: "+1 (952) 533-2258".into(),
                friends: Friends::default().into(),
                favorite_fruit: "banana".into(),
            }
        }
    }

    #[derive(Serialize)]
    pub struct Friends(Vec<Friend>);

    #[derive(Serialize)]
    pub struct Friend {
        id: i32,
        name: String,
    }

    impl Default for Friends {
        fn default() -> Self {
            Friends(vec![
                Friend {
                    id: 0,
                    name: "Colon Salazar".into(),
                },
                Friend {
                    id: 1,
                    name: "French Mcneil".into(),
                },
                Friend {
                    id: 2,
                    name: "Carol Martin".into(),
                },
            ])
        }
    }

    impl From<Friends> for Vec<Friend> {
        fn from(f: Friends) -> Self {
            f.0
        }
    }

    #[test]
    fn test_proc_macro() {
        let lex: Vec<LexOperator> = precompile_lex!(.metadata[1,2,4-6,hello]);
        println!("{:?}", lex);
    }

    #[test]
    fn test_query() {
        let lex = precompile_lex!(.metadata[1,2,4-6,hello]);
        let data = User::default();
        let query_res = query(data, lex);
        println!("{:?}", query_res.unwrap());
    }
}
