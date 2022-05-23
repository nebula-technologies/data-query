use crate::lexer_constants::*;

pub struct Identifier {
    name: Option<String>,
    generic: Option<GenericObjectIndex>,
}

pub enum GenericObjectIndex {
    Wildcard,
    Slice(Vec<Slicer>)
}

pub enum Slicer {
    Index(usize),
    Slice(usize,usize),
    Ident(String)
}

pub enum LexOperator{
    Identifier(Identifier),
    Pipe(Vec<LexOperator>),
}


pub fn compile(s: &str) -> Vec<LexOperator> {
    let mut last = "";
    for c in s.chars() {
        match c {
            LEX_IDENTIFIER => {

            }
            _ => {

            }
        }
    }

    Vec::new()
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