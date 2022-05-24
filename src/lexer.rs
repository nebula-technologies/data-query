use crate::lexer_constants::*;
use std::num::ParseIntError;

pub enum LexerError {
    EndOfQuery { expected: String },
    FailedToParseInt(ParseIntError),
}

impl From<ParseIntError> for LexerError {
    fn from(e: ParseIntError) -> Self {
        Self::FailedToParseInt(e)
    }
}

pub type LexResult<T> = Result<T, LexerError>;

pub enum GenericObjectIndex {
    Wildcard,
    Slice(Vec<Slicer>),
}

pub enum Slicer {
    Index(usize),
    SliceFrom(usize),
    SliceTo(usize),
    Ident(String),
}

pub enum LexOperator {
    Identifier(String),
    Pipe(Vec<LexOperator>),
    Generic(GenericObjectIndex),
}

pub fn compile(s: &str) -> Vec<LexOperator> {
    let lexer_vec = s.chars().into_iter().collect::<Vec<char>>();
    let mut lex = Vec::new();
    let mut last = "";
    let mut generic = "";

    Vec::new()
}

pub fn generic_compiler(
    mut lexer_vec: &Vec<char>,
    mut collect: String,
) -> LexResult<Vec<LexOperator>> {
    let char = lexer_vec.pop();
    let mut lex_complied = Vec::new();
    if let Some(c) = char {
        match c {
            LEX_IDENTIFIER => {
                lex_complied.push(LexOperator::Identifier(collect.to_string()));
                collect = Default::default();
            }
            LEX_GENERIC_START => {
                let v = generic_object_index(lexer_vec, Default::default(), Vec::new())?;
                lex_complied.push(LexOperator::Generic(v));
            }
            _ => collect.push(c),
        }
    }

    Ok(lex_complied)
}

fn generic_object_index(
    mut lexer_vec: &Vec<char>,
    collect: String,
    mut slicer: Vec<Slicer>,
) -> LexResult<GenericObjectIndex> {
    let char = lexer_vec.pop();
    if let Some(c) = char {
        match c {
            LEX_GENERIC_END => {
                if collect.is_empty() && slicer.is_empty() {
                    Ok(GenericObjectIndex::Wildcard)
                } else if !collect.is_empty() {
                    if let Some(Slicer::SliceFrom(u)) = slicer.last() {
                        let slice = collect.parse::<usize>().map_err(LexerError::from)?;
                        slicer.push(Slicer::SliceTo(slice))
                    }
                    Ok(GenericObjectIndex::Slice(slicer))
                } else {
                    Ok(GenericObjectIndex::Slice(slicer))
                }
            }
            _ => {}
        }
    } else {
        Err(LexerError::EndOfQuery {
            expected: String::from(LEX_GENERIC_END),
        })
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
