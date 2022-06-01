extern crate data_query_lexical;
#[macro_use]
extern crate data_query_proc;
extern crate railsgun;
extern crate regex;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;

mod error;

pub use crate::error::QueryError;
use data_query_lexical::{GenericObjectIndex, LexOperator, LexicalOperations, Slicer};

use serde::Serialize;
use serde_json::{Map, Value};
use std::cmp::Ordering;

/// Alias for a `Result` with the error type `serde_json::Error`.
pub type QueryResult<T> = std::result::Result<T, QueryError>;

struct ComType {
    usize: Option<usize>,
    string: Option<String>,
}

impl PartialOrd<Self> for ComType {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        use Ordering::*;
        if let (Some(s), Some(o)) = (self.usize, other.usize) {
            if s > o {
                return Some(Greater);
            } else if s < o {
                return Some(Less);
            } else if s == o {
                return Some(Equal);
            }
        }
        None
    }
}

// impl Ord for ComType {
//     fn cmp(&self, other: &Self) -> Ordering {
//         todo!()
//     }
// }

impl Eq for ComType {}

impl PartialEq for ComType {
    fn eq(&self, other: &Self) -> bool {
        if let (Some(s), Some(o)) = (self.usize, other.usize) {
            s == o
        } else {
            false
        }
    }
}

impl From<usize> for ComType {
    fn from(u: usize) -> Self {
        Self {
            usize: Some(u),
            string: Some(format!("{}", u)),
        }
    }
}
impl From<&usize> for ComType {
    fn from(u: &usize) -> Self {
        Self {
            usize: Some(*u),
            string: Some(format!("{}", u)),
        }
    }
}

impl From<String> for ComType {
    fn from(s: String) -> Self {
        Self {
            usize: s.parse::<usize>().ok(),
            string: Some(s),
        }
    }
}

impl From<&str> for ComType {
    fn from(s: &str) -> Self {
        Self::from(s.to_string())
    }
}

impl From<&mut usize> for ComType {
    fn from(u: &mut usize) -> Self {
        Self::from(u.clone())
    }
}

pub fn query<S: Serialize, Q: TryInto<LexicalOperations>>(
    s: S,
    query: Q,
) -> QueryResult<Vec<Value>> {
    let mut lexes = query
        .try_into()
        // TODO: This error handling needs to be fixed!
        .map_err(|_e| QueryError::from(format!("Gulp")))?;
    let data = serde_json::to_value(s).map_err(QueryError::from)?;
    let mut results = Vec::new();
    query_processor(&data, &mut lexes, &mut results, 0)?;
    Ok(results)
}

fn query_processor(
    data: &Value,
    query: &mut LexicalOperations,
    results: &mut Vec<Value>,
    mut depth: usize,
) -> QueryResult<()> {
    depth += 1;
    println!("current depth: {}", depth);
    if query.is_empty() {
        results.push(data.clone());
        return Ok(());
    } else {
        let key_query = query
            .pop_front()
            .ok_or(QueryError::UncontrolledError("Empty".to_string()))?;
        match data {
            Value::Array(v) => {
                println!("Data Type - Array - {}", line!());
                match key_query.clone() {
                    LexOperator::Identifier(ident) => {
                        println!("Key Index - Identifier");
                        println!("data: {:?}", v);
                        println!("key: {:?}", ident);
                        return if let Ok(i) = ident.parse::<usize>() {
                            query_processor(&v[i], query, results, depth)
                        } else {
                            Err(QueryError::CannotUseIdentifierAsArrayKeyIndex(ident))
                        };
                    }
                    LexOperator::Pipe(_p) => {
                        todo!();
                    }
                    LexOperator::Generic(mut g) => {
                        println!("Key Index - Generic");
                        println!("data: {:?}", v);
                        println!("key: {:?}", g);
                        return query_slice_w_generic_object_index(
                            &v, &mut g, query, results, depth,
                        );
                    }
                }
            }
            Value::Object(m) => {
                println!("Data Type - Object");
                println!("{:?}", key_query);
                match key_query {
                    LexOperator::Identifier(ident) => {
                        println!("Key Index - Identifier");
                        println!("data: {:?}", m);
                        println!("key: {:?}", ident);
                        return if m.contains_key(&ident) {
                            if let Some(value) = m.get(&*ident) {
                                query_processor(value, query, results, depth)
                            } else {
                                Ok(())
                            }
                        } else {
                            Err(QueryError::CannotUseIdentifierAsArrayKeyIndex(ident))
                        };
                    }
                    LexOperator::Pipe(_p) => {
                        todo!();
                    }
                    LexOperator::Generic(mut g) => {
                        println!("Key Index - Generic");
                        println!("data: {:?}", m);
                        println!("key: {:?}", g);
                        return query_map_w_generic_object_index(m, &mut g, query, results, depth);
                    }
                }
            }
            _ => {
                println!("Unknown");
                return Ok(());
            }
        }
    }
    Ok(())
}

fn query_slice_w_generic_object_index(
    data: &Vec<Value>,
    index_match: &mut GenericObjectIndex,
    query: &mut LexicalOperations,
    results: &mut Vec<Value>,
    depth: usize,
) -> QueryResult<()> {
    for (k, v) in data.iter().enumerate() {
        println!("match key: {}; value: {};", k, v);
        if match_slice_to_key(&format!("{}", k), index_match) {
            println!("Slice match - {} == {:?}", k, index_match);
            query_processor(v, query, results, depth)?
        } else {
            println!("Slice No match - {} == {:?}", k, index_match);
        }
    }
    Ok(())
}

fn query_map_w_generic_object_index(
    data: &Map<String, Value>,
    index_match: &mut GenericObjectIndex,
    query: &mut LexicalOperations,
    results: &mut Vec<Value>,
    depth: usize,
) -> QueryResult<()> {
    for (k, v) in data.iter() {
        if match_slice_to_key(&format!("{}", k), index_match) {
            query_processor(v, query, results, depth)?
        }
    }
    Ok(())
}

fn key_match_map(key: &String, query: &LexOperator) -> bool {
    match query {
        LexOperator::Identifier(ident) => {
            if key == ident {
                true
            } else {
                false
            }
        }
        LexOperator::Pipe(_p) => {
            todo!();
        }
        LexOperator::Generic(g) => {
            return match_slice_to_key(key, &mut g.clone());
        }
    }
}

fn match_slice_to_key(key: &str, query: &mut GenericObjectIndex) -> bool {
    println!("key: {}; query: {:?};", key, query);
    let key_comp: ComType = key.into();
    match query {
        GenericObjectIndex::Wildcard => true,
        GenericObjectIndex::Slice(slice) => {
            println!("Slicer array");
            for s in slice {
                match s {
                    Slicer::Index(i) => {
                        println!("here");
                        if key_comp == ComType::from(i) {
                            println!("success");
                            return true;
                        } else {
                            println!("Failed");
                        }
                    }
                    Slicer::Slice(f, t) => {
                        if key_comp <= ComType::from(f) && key_comp >= ComType::from(t) {
                            return true;
                        }
                    }
                    Slicer::Ident(ident) => {
                        if let Ok(ref i) = ident.parse::<usize>() {
                            if key_comp == ComType::from(i) {
                                return true;
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
    use crate::{query, ComType};
    use data_query_lexical::{compile, LexOperator};
    use serde_derive::Serialize;
    use std::collections::LinkedList;

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
    fn test_com_type() {
        let large = ComType::from(100);
        let mid = ComType::from(50);
        let low = ComType::from(10);
        assert!(large == large);
        assert!(mid < large);
        assert!(low < mid);
    }

    #[test]
    fn test_proc_macro() {
        let lex: LinkedList<LexOperator> = precompile_lex!(.metadata[1,2,4-6,hello]);
        println!("{:?}", lex);
    }

    #[test]
    fn test_query_precompile() {
        let lex = precompile_lex!(.friends[1].name);
        println!("{:?}", lex);
        let data = User::default();
        let query_res = query(data, lex);
        println!("{:?}", query_res.unwrap());
    }

    #[test]
    fn test_query_multiple_results_precompile() {
        let lex = precompile_lex!(.friends[1,2].name);
        println!("{:?}", lex);
        let data = User::default();
        let query_res = query(data, lex);
        println!("{:?}", query_res.unwrap());
    }

    #[test]
    fn test_query() {
        let lex = compile(".friends[1].name").unwrap();
        println!("{:?}", lex);
        let data = User::default();
        let query_res = query(data, lex);
        println!("{:?}", query_res.unwrap());
    }

    #[test]
    fn test_query_multiple_results() {
        let lex = compile(".friends[1,2].name").unwrap();
        println!("{:?}", lex);
        let data = User::default();
        let query_res = query(data, lex);
        println!("{:?}", query_res.unwrap());
    }
}
