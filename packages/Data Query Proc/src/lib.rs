use data_query_lexer as lexer;
use data_query_lexer::MacroFormat;
use proc_macro::TokenStream;
use std::str::FromStr;

#[proc_macro]
pub fn precompile_lex(input: TokenStream) -> TokenStream {
    let lex = input.to_string();
    println!("{}", lex);
    let const_lex = lexer::compile(&lex);
    if let Err(v) = const_lex {
        panic!(
            "It was not possible to create a const value to the expexted lexica string: {:?}",
            v
        )
    }
    let code = const_lex.unwrap().macro_fmt();
    TokenStream::from_str(code.as_str()).unwrap()
}
