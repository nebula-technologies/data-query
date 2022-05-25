use data_query_lexer;
use proc_macro::TokenStream;

#[proc_macro]
pub fn api_swag(input: TokenStream) -> TokenStream {
    let lex = input.to_string();
    let const_lex = data_query_lexer::lexer::compile(&lex);
    if let Err(v) = const_lex {
        panic!(
            "It was not possible to create a const value to the expexted lexica string: {:?}",
            const_lex.unwrap_err()
        )
    }
    TokenStream::from_str()
}

trait MacroFormat {
    fn macro_fmt() -> String {
        "hello".to_string()
    }
}
