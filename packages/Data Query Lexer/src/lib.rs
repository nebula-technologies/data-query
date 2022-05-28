mod lexer;
pub(crate) mod lexer_constants;
pub use lexer::*;

pub trait MacroFormat {
    fn macro_fmt(&self) -> String;
}

impl MacroFormat for lexer::Slicer {
    fn macro_fmt(&self) -> String {
        match self {
            Slicer::Index(i) => format!("::data_query_lexer::Slicer::Index({})", i),
            Slicer::Slice(f, t) => format!("::data_query_lexer::Slicer::Slice({},{})", f, t),
            Slicer::Ident(i) => {
                format!("::data_query_lexer::Slicer::Ident(\"{}\".into())", i)
            }
        }
    }
}

impl MacroFormat for Vec<lexer::LexOperator> {
    fn macro_fmt(&self) -> String {
        format!(
            "vec![{}]",
            self.iter()
                .map(|t| t.macro_fmt())
                .collect::<Vec<String>>()
                .join(",")
        )
    }
}

impl MacroFormat for lexer::LexOperator {
    fn macro_fmt(&self) -> String {
        match self {
            LexOperator::Identifier(i) => {
                format!(
                    "::data_query_lexer::LexOperator::Identifier(\"{}\".into())",
                    i
                )
            }
            LexOperator::Pipe(p) => p.macro_fmt(),
            LexOperator::Generic(g) => format!(
                "::data_query_lexer::LexOperator::Generic({})",
                g.macro_fmt()
            ),
        }
    }
}

impl MacroFormat for lexer::GenericObjectIndex {
    fn macro_fmt(&self) -> String {
        match self {
            GenericObjectIndex::Wildcard => {
                format!("::data_query_lexer::GenericObjectIndex::Wildcard")
            }
            GenericObjectIndex::Slice(s) => format!(
                "::data_query_lexer::GenericObjectIndex::Slice(vec![{}])",
                s.iter()
                    .map(|s| s.macro_fmt())
                    .collect::<Vec<String>>()
                    .join(",")
            ),
        }
    }
}
