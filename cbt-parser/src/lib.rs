use cbt_common::{
    words::{Word, Words},
    Context, Token, TokenKind,
};
use std::iter::Peekable;

#[derive(Debug)]
pub struct Parser<'a> {
    pub source: Peekable<std::vec::IntoIter<Token<'a>>>,
    pub src_len: usize,
    pub ctx: Context<'a>,
    pub idx: usize,
}

impl<'a> Parser<'a> {
    pub fn init(source: Vec<Token<'a>>, ctx: Context<'a>) -> Self {
        let len = source.len();
        Self {
            source: source.into_iter().peekable(),
            src_len: len,
            ctx,
            idx: 0,
        }
    }

    pub fn verify_file_prefix(&mut self) {
        if self.src_len < 4 {
            panic!("Not enough tokens.");
        }
        self.expect_and_skip_crash(vec![
            TokenKind::Word(Word {
                which: Words::Begin,
                plural: false,
            }),
            TokenKind::Word(Word {
                which: Words::Program,
                plural: false,
            }),
            TokenKind::Period,
        ]);
    }

    pub fn expect_and_skip_crash(&mut self, expectance: Vec<TokenKind>) {
        if !self.expect_and_skip(expectance) {
            println!("Call to `expect_and_skip_crash`");
            std::process::exit(1);
        }
    }

    pub fn expect_and_skip(&mut self, expectance: Vec<TokenKind>) -> bool {
        for e in expectance {
            if !self.expect(e) {
                return false;
            }
            self.source.next();
        }
        true
    }

    pub fn expect(&mut self, expected: TokenKind) -> bool {
        let Some(next) = self.source.peek() else {
            return false;
        };
        if next.inner != expected {
            return false;
        }
        true
    }
}
