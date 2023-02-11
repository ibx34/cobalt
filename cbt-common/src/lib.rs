pub mod errors;
pub mod parser;
pub mod words;
use std::ops::Range;

#[derive(Debug)]
/// This struct is kinda useless rightnow.
pub struct Context<'a> {
    pub file: &'a str,
    pub source: Vec<char>,
}

#[derive(Debug, PartialEq, Eq)]
/// A struct that holds information about where a token can be.
pub struct Location<'a> {
    /// The file where this location is relative to
    pub file: Option<&'a str>,
    /// How many characters from the start this token is
    pub offset: usize,
    /// The line the token is on. This may or may not be used.
    pub line: usize,
    /// How many characters the token spans.
    pub range: Option<Range<usize>>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum TokenKind {
    Colon,
    String,
    Period,
    Word(words::Word),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Token<'a> {
    pub inner: TokenKind,
    pub location: Location<'a>,
}
