use crate::lexer_constants::*;
use std::num::ParseIntError;

#[derive(Debug, Eq, PartialEq)]
pub enum LexerError {
    EndOfQuery {
        expected: String,
        char_pointer: usize,
        lex: String,
    },
    FailedToParseInt(ParseIntError),
    UnexpectedCharacter {
        expected: String,
        found: String,
        char_pointer: usize,
        lex: String,
    },
}

impl From<ParseIntError> for LexerError {
    fn from(e: ParseIntError) -> Self {
        Self::FailedToParseInt(e)
    }
}

pub type LexResult<T> = Result<T, LexerError>;

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum GenericObjectIndex {
    Wildcard,
    Slice(Vec<Slicer>),
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Slicer {
    Index(usize),
    SliceFrom(usize),
    SliceTo(usize),
    Ident(String),
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum LexOperator {
    Identifier(String),
    Pipe(Vec<LexOperator>),
    Generic(GenericObjectIndex),
}

pub fn compile(s: &str) -> LexResult<Vec<LexOperator>> {
    let mut lexer_vec = s.chars().into_iter().collect::<Vec<char>>();
    lexer_vec.reverse();
    generic_compiler(
        &mut lexer_vec,
        &mut Default::default(),
        Default::default(),
        Default::default(),
    )
}

pub fn generic_compiler(
    lexer_vec: &mut Vec<char>,
    mut operator: &mut Vec<LexOperator>,
    mut collect: String,
    mut char_pointer: usize,
) -> LexResult<Vec<LexOperator>> {
    let char = lexer_vec.pop();
    if let Some(c) = char {
        char_pointer = char_pointer + 1;
        match c {
            LEX_IDENTIFIER => {
                if !collect.is_empty() {
                    operator.push(LexOperator::Identifier(collect));
                }
                collect = Default::default();
            }
            LEX_GENERIC_START => {
                if !collect.is_empty() {
                    operator.push(LexOperator::Identifier(collect));
                    collect = Default::default();
                }
                let v =
                    generic_object_index(lexer_vec, Default::default(), Vec::new(), char_pointer)?;
                operator.push(LexOperator::Generic(v));
            }
            _ => collect.push(c),
        }
        generic_compiler(lexer_vec, operator, collect, char_pointer)
    } else {
        Ok(operator.clone())
    }
}

fn generic_object_index(
    lexer_vec: &mut Vec<char>,
    mut collect: String,
    mut slicer: Vec<Slicer>,
    mut char_pointer: usize,
) -> LexResult<GenericObjectIndex> {
    let char = lexer_vec.pop();
    if let Some(c) = char {
        char_pointer += 1;
        match c {
            LEX_GENERIC_END => {
                if collect.is_empty() && slicer.is_empty() {
                    Ok(GenericObjectIndex::Wildcard)
                } else if !collect.is_empty() {
                    if let Some(Slicer::SliceFrom(u)) = slicer.last() {
                        let slice = collect.parse::<usize>().map_err(LexerError::from)?;
                        slicer.push(Slicer::SliceTo(slice))
                    } else if let Ok(u) = collect.parse::<usize>() {
                        slicer.push(Slicer::Index(u));
                    } else {
                        slicer.push(Slicer::Ident(collect.clone()));
                    }
                    Ok(GenericObjectIndex::Slice(slicer))
                } else {
                    Ok(GenericObjectIndex::Slice(slicer))
                }
            }
            LEX_GENERIC_SEPARATOR => {
                if collect.is_empty() && slicer.is_empty() {
                    Err(LexerError::UnexpectedCharacter {
                        expected: "Integer/String".to_string(),
                        found: LEX_GENERIC_SEPARATOR.to_string(),
                        char_pointer,
                        lex: format!("{:?}", lexer_vec),
                    })
                } else {
                    if let Some(Slicer::SliceFrom(u)) = slicer.last() {
                        let slice = collect.parse::<usize>().map_err(LexerError::from)?;
                        slicer.push(Slicer::SliceTo(slice))
                    } else if let Ok(u) = collect.parse::<usize>() {
                        slicer.push(Slicer::Index(u));
                    } else {
                        slicer.push(Slicer::Ident(collect.clone()));
                    }
                    collect = Default::default();
                    generic_object_index(lexer_vec, collect, slicer, char_pointer)
                }
            }
            LEX_GENERIC_SLICE => {
                if collect.is_empty() && slicer.is_empty() {
                    return Err(LexerError::UnexpectedCharacter {
                        expected: "Integer/String".to_string(),
                        found: LEX_GENERIC_SEPARATOR.to_string(),
                        char_pointer,
                        lex: format!("{:?}", lexer_vec),
                    });
                } else if let Ok(u) = collect.parse::<usize>() {
                    slicer.push(Slicer::SliceFrom(u));
                } else {
                    return Err(LexerError::UnexpectedCharacter {
                        expected: "Integer".to_string(),
                        found: "String".to_string(),
                        char_pointer,
                        lex: format!("{:?}", lexer_vec),
                    });
                }
                collect = Default::default();
                generic_object_index(lexer_vec, collect, slicer, char_pointer)
            }
            _ => {
                collect.push(c);
                generic_object_index(lexer_vec, collect, slicer, char_pointer)
            }
        }
    } else {
        Err(LexerError::EndOfQuery {
            expected: String::from(LEX_GENERIC_END),
            char_pointer,
            lex: format!("{:?}", lexer_vec),
        })
    }
}

#[cfg(test)]
mod test {
    use crate::lexer::GenericObjectIndex::*;
    use crate::lexer::LexOperator::*;
    use crate::lexer::Slicer::*;
    use crate::lexer::{
        compile, generic_compiler, generic_object_index, GenericObjectIndex, LexOperator,
        LexResult, Slicer,
    };

    fn lex_vec(s: &str) -> Vec<char> {
        s.chars().into_iter().collect::<Vec<char>>()
    }

    #[test]
    pub fn test_slicer() {
        let mut lex_vec = lex_vec("1,2,4-6,hello]");
        lex_vec.reverse();
        let slicer = generic_object_index(&mut lex_vec, "".to_string(), Vec::new(), 0usize);
        let true_generic_object = GenericObjectIndex::Slice(vec![
            Slicer::Index(1),
            Slicer::Index(2),
            Slicer::SliceFrom(4),
            Slicer::SliceTo(6),
            Ident("hello".to_string()),
        ]);

        println!("{:?}", slicer);
        assert_eq!(true_generic_object, slicer.unwrap())
    }

    #[test]
    pub fn test_generic_compiler() {
        let mut lex_vec = lex_vec(".metadata[1,2,4-6,hello]");
        lex_vec.reverse();
        let mut operator = Vec::new();
        let compiled_lex = generic_compiler(
            &mut lex_vec,
            &mut operator,
            Default::default(),
            Default::default(),
        );
        let true_result: LexResult<Vec<LexOperator>> = Ok(vec![
            Identifier("metadata".to_string()),
            Generic(Slice(vec![
                Index(1),
                Index(2),
                SliceFrom(4),
                SliceTo(6),
                Ident("hello".to_string()),
            ])),
        ]);
        println!("{:?}", compiled_lex);
        assert_eq!(true_result, compiled_lex);
    }

    #[test]
    pub fn test_compiler() {
        let compiled_lex = compile(".metadata[1,2,4-6,hello]");
        let true_result: LexResult<Vec<LexOperator>> = Ok(vec![
            Identifier("metadata".to_string()),
            Generic(Slice(vec![
                Index(1),
                Index(2),
                SliceFrom(4),
                SliceTo(6),
                Ident("hello".to_string()),
            ])),
        ]);
        println!("{:?}", compiled_lex);
        assert_eq!(true_result, compiled_lex);
    }
}

/*


{
  "default": "Personal",
  "annotation-field": "annotations",
  "workspaces": {
    "Personal": {
      "HelloWorld": {
        "annotations": {
          "my-app.io/group": "HelloWorld"
        }
      },
      "NoWorld": {}
    }
  }
}



 */
