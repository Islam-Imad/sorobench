//! Lexer and parser for the // ---- expectation DSL, ported from solc's
//! TestFileParser. Entry point is parse_calls; the splitter that produces the
//! raw block text lives in crate::testfile.

pub mod ast;
pub mod parser;
pub mod scanner;

#[cfg(test)]
mod tests;

use std::collections::HashSet;
use std::fmt;

pub use ast::*;

// A parse error (message only), like solc's TestParserError.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub message: String,
}

impl ParseError {
    pub fn new(message: impl Into<String>) -> ParseError {
        ParseError {
            message: message.into(),
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for ParseError {}

pub type PResult<T> = Result<T, ParseError>;

// Framework-builtin signatures the parser recognizes. Builtins are checked
// while parsing, so a bare identifier like storageEmpty is legal and tagged
// Kind::Builtin.
pub type Builtins = HashSet<String>;

// The builtins used by the solc semantic-test corpus (from solc's
// SemanticTest::makeBuiltins).
pub fn semantic_test_builtins() -> Builtins {
    [
        "isoltest_builtin_test",
        "isoltest_side_effects_test",
        "balance",
        "storageEmpty",
        "account",
    ]
    .into_iter()
    .map(String::from)
    .collect()
}

// Parse a raw // ---- expectation block into its function calls. source is the
// block text from the region splitter (the //-prefixed lines after the // ----
// marker); builtins is the recognized builtin set.
pub fn parse_calls(source: &str, builtins: &Builtins) -> PResult<Vec<FunctionCall>> {
    parser::Parser::new(source, builtins).parse_function_calls(0)
}

// Like parse_calls but seeds error line numbers with line_offset.
pub fn parse_calls_at(
    source: &str,
    builtins: &Builtins,
    line_offset: usize,
) -> PResult<Vec<FunctionCall>> {
    parser::Parser::new(source, builtins).parse_function_calls(line_offset)
}
