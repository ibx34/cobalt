use cbt_common::{
    errors::Error,
    words::{Word, Words},
    Context, Location, Token, TokenKind,
};
use std::{iter::Peekable, ops::Range, str::Chars};

#[derive(Debug)]
pub struct Lexer<'a> {
    pub source: Peekable<Chars<'a>>,
    pub idx: usize,
    pub ctx: Context<'a>,
    pub tokens: Vec<Token<'a>>,
    pub line: usize,
}

impl<'a> Lexer<'a> {
    pub fn init(source: &'a str, file_name: &'a str) -> Self {
        let chars = source.chars();
        let context = Context {
            file: file_name,
            source: chars.clone().collect::<Vec<char>>(),
        };

        let chars = chars.clone().into_iter().peekable();

        Self {
            source: chars,
            ctx: context,
            idx: 0,
            line: 0,
            tokens: Vec::new(),
        }
    }

    pub fn lex_all(&mut self) -> Result<(), Error> {
        let file_len = self.ctx.source.len();
        while self.idx < file_len {
            self.lex()?;
        }
        Ok(())
    }

    pub fn push_back(
        &mut self,
        token_kind: TokenKind,
        range: Option<Range<usize>>,
        advance: bool,
    ) -> Result<(), Error> {
        let location = Location {
            file: None,
            offset: self.idx,
            line: self.line,
            range: range,
        };
        self.tokens.push(Token {
            inner: token_kind,
            location,
        });

        if advance {
            self.advance();
        }
        Ok(())
    }

    pub fn lex(&mut self) -> Result<(), Error> {
        let Some(current) = self.source.peek() else {
            return Err(Error::new(String::from("Failed to peek next character")))
        };
        match current {
            ':' => self.push_back(TokenKind::Colon, None, true),
            '.' => self.push_back(TokenKind::Period, None, true),
            ' ' => {
                self.advance();
                return Ok(());
            }
            '\n' => {
                self.advance();
                self.line += 1;
                return Ok(());
            }
            '"' => {
                self.advance();
                let current = self.idx.clone();
                while let Some(next) = self.source.peek() {
                    match next {
                        '"' => break,
                        '\n' => {
                            self.line += 1;
                            break;
                        }
                        _ => self.advance(),
                    }
                }
                self.push_back(TokenKind::String, Some(current..self.idx), true)
            }
            _ => {
                if ('a'..='z').contains(current) || ('A'..='Z').contains(current) {
                    let current = self.idx.clone();
                    while let Some(next) = self.source.peek() {
                        if *next == ' ' || !next.is_alphabetic() {
                            break;
                        }
                        self.advance();
                    }
                    let Some(keyword) = self.ctx.source.get(current..self.idx) else {
                        return Err(Error::new(String::from("Failed to collect string from source.")));
                    };
                    let keyword = keyword.iter().collect::<String>();
                    if let Ok(word) = keyword.to_lowercase().as_str().try_into() {
                        self.push_back(
                            TokenKind::Word(Word {
                                which: word,
                                plural: false,
                            }),
                            Some(current..self.idx),
                            false,
                        )
                    } else {
                        return Err(Error::new(String::from(
                            "Potential keyword didn't match any known keywords",
                        )));
                    }
                } else {
                    return Err(Error::new(String::from("Unsupported character")));
                }
            }
        }
    }

    pub fn advance(&mut self) {
        self.idx += 1;
        self.source.next();
    }
}
