extern crate railsgun;
extern crate regex;
extern crate serde;
extern crate serde_json;

use crate::error::Error;

mod error;
mod lexer;
mod lexer_constants;

pub use lexer_constants::*;

/// Alias for a `Result` with the error type `serde_json::Error`.
pub type Result<T> = std::result::Result<T, Error>;
