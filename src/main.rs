use std::ops::Range;

use errors::ErrorClient;

pub mod ast;
pub mod cg;
pub mod errors;
pub mod node;
pub mod p;

// Non-plural list of words. Some of these may be plural, or end an S, which will be handled later on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Words {
    Define,
    Module,
    Function,
    Call,
    Equal,
    Argument,
    The,
    With,
    Contents,
    End,
    Is,
    To,
    Set,
    A,
    Expects,
    That,
    Returns,
    Contains,
    Display,
    If,
    Then,
    Do,
}

impl TryFrom<&str> for Words {
    type Error = String;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "define" => Ok(Self::Define),
            "module" => Ok(Self::Module),
            "function" => Ok(Self::Function),
            "call" => Ok(Self::Call),
            "equal" => Ok(Self::Equal),
            "argument" => Ok(Self::Argument),
            "the" => Ok(Self::The),
            "with" => Ok(Self::With),
            "contents" => Ok(Self::Contents),
            "contains" => Ok(Self::Contains),
            "end" => Ok(Self::End),
            "is" => Ok(Self::Is),
            "to" => Ok(Self::To),
            "set" => Ok(Self::Set),
            "a" => Ok(Self::A),
            "expects" => Ok(Self::Expects),
            "that" => Ok(Self::That),
            "returns" => Ok(Self::Returns),
            "display" => Ok(Self::Display),
            "if" => Ok(Self::If),
            "then" => Ok(Self::Then),
            "do" => Ok(Self::Do),
            _ => Err(String::from("Ye bad")),
        }
    }
}

impl From<Words> for String {
    fn from(src: Words) -> String {
        match src {
            Words::Define => "define",
            Words::Module => "module",
            Words::Function => "function",
            Words::Call => "call",
            Words::Equal => "equal",
            Words::Argument => "argument",
            Words::The => "the",
            Words::With => "with",
            Words::Contents => "contents",
            Words::End => "end",
            Words::Is => "is",
            Words::To => "to",
            Words::Set => "set",
            Words::A => "a",
            Words::Expects => "expects",
            Words::That => "that",
            Words::Contains => "contains",
            Words::Returns => "returns",
            Words::Display => "display",
            Words::If => "if",
            Words::Then => "then",
            Words::Do => "do",
        }
        .to_ascii_uppercase()
    }
}

// TODO: plural word detection.

#[derive(Debug, PartialEq, Eq)]
pub struct Word {
    pub which: Words,
    // Just so stuff later on knows to check!
    pub plural: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub struct TokenLoc {
    pub idx: usize,
    pub span: Range<usize>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Token {
    pub inner: Tokens,
    pub location: TokenLoc,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Tokens {
    Word(Word),
    SemiColon,
    Colon,
    String,
    DollarSign,
    Period,
}

impl ToString for Tokens {
    fn to_string(&self) -> String {
        match self {
            Tokens::SemiColon => ";".to_string(),
            Tokens::Colon => ":".to_string(),
            Tokens::DollarSign => "$".to_string(),
            Tokens::Period => ".".to_string(),
            Tokens::Word(word) => word.which.clone().into(),
            _ => "Unable to turn into string.".to_string(),
        }
    }
}

pub struct Lexer {
    pub source: Vec<char>,
    pub idx: usize,
    pub results: ast::AST,
}

impl Lexer {
    pub fn lex(&mut self) {
        let Some(current) = self.current() else {
            panic!("Failed to get current character @ index #{}", self.idx);
        };
        match current {
            ':' => self.push_back(Tokens::Colon, None),
            ';' => self.push_back(Tokens::SemiColon, None),
            '$' => self.push_back(Tokens::DollarSign, None),
            '.' => self.push_back(Tokens::Period, None),
            '"' => {
                let current_idx = self.idx;
                while let Some(next) = self.peek(None) {
                    if next == '"' {
                        break;
                    }
                    self.advance(None);
                }
                self.push_back(Tokens::String, Some(current_idx + 1..self.idx + 1));
                self.advance(None);
            }
            _ => {
                if ('a'..='z').contains(current) || ('A'..='Z').contains(current) {
                    let current_idx = self.idx;
                    while let Some(next) = self.peek(None) {
                        if next == ' ' || !next.is_alphabetic() {
                            break;
                        }
                        self.advance(None);
                    }
                    let Some(keyword) = self.source.get(current_idx..self.idx + 1) else {
                        panic!("Failed to collect keyword from the source.");
                    };
                    let keyword = keyword.iter().collect::<String>();
                    if let Ok(word) = keyword.to_lowercase().as_str().try_into() {
                        self.push_back(
                            Tokens::Word(Word {
                                which: word,
                                plural: false,
                            }),
                            Some(current_idx..self.idx),
                        );
                    } else {
                        let mut error = ErrorClient::new("0002", crate::errors::MessageKind::ERROR);
                        error.end_process(true);
                        // TODO: The parser should have a context field that is aware of this kind of stuff.
                        error.set_file(
                            "module_level_func.cbt",
                            "./tests/pass/module_level_func.cbt",
                        );
                        error.set_span(current_idx..self.idx + 1);
                        let note = format!("This is not a valid keyword");
                        error.add_label(Some(&note));
                        error.build_and_emit();
                        return;
                    }
                }
            }
        }
    }

    pub fn push_back(&mut self, token: Tokens, range: Option<Range<usize>>) {
        self.results.ast.push(Token {
            inner: token,
            location: TokenLoc {
                idx: self.idx,
                span: range.unwrap_or(self.idx..self.idx),
            },
        });
    }

    pub fn lex_all(&mut self) {
        while !self.eof(None) {
            self.lex();
            self.advance(None);
        }
    }

    pub fn current(&self) -> Option<&char> {
        self.source.get(self.idx)
    }
    pub fn eof(&self, amount: Option<usize>) -> bool {
        let idx = amount.unwrap_or(self.idx);

        if self.source.get(idx).is_some() {
            return false;
        }
        true
    }
    pub fn advance(&mut self, amount: Option<usize>) -> bool {
        let amount = amount.unwrap_or(1);

        if !self.eof(Some(amount)) {
            self.idx += amount;
            return true;
        }
        false
    }
    pub fn peek(&self, amount: Option<usize>) -> Option<char> {
        self.source.get(self.idx + amount.unwrap_or(1)).copied()
    }
}

fn main() {
    let input_str = std::fs::read_to_string("tests/pass/module_level_func.cbt").unwrap();

    let mut lexer = Lexer {
        source: input_str.chars().collect(),
        results: ast::AST { ast: vec![] },
        idx: 0,
    };

    lexer.lex_all();

    let mut parser = p::Parser {
        source: lexer.results.ast.into_iter().peekable(),
        idx: 0,
        nodes: vec![],
        source_str: lexer.source,
    };
    parser.parse();
    unsafe {
        //cg::codegen(parser.nodes);
        let mut codegen = cg::CodeGen::init(parser.nodes.into_iter().peekable());
        codegen.setup_main_module();
        let func = codegen.stmts.next().unwrap();
        codegen.visit_fn(func);
        let func = codegen.stmts.next().unwrap();
        codegen.visit_fn(func);
        codegen.verify_and_dump();
    }
}
