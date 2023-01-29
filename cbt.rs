// abstract syntax tree
pub mod ast;
// The parser
pub mod p;
// Nodes
pub mod node;

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
            "end" => Ok(Self::End),
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
        }
        .to_ascii_uppercase()
    }
}

// TODO: plural word detection.

#[derive(Debug)]
pub struct Word {
    pub which: Words,
    // Just so stuff later on knows to check!
    pub plural: bool,
}

#[derive(Debug)]
pub enum Tokens {
    Word(Word),
    SemiColon,
    Colon,
    String(String),
    DollarSign,
    Period,
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
            ':' => self.results.ast.push(Tokens::Colon),
            ';' => self.results.ast.push(Tokens::SemiColon),
            '$' => self.results.ast.push(Tokens::DollarSign),
            '.' => self.results.ast.push(Tokens::Period),
            '"' => {
                let current_idx = self.idx;
                while let Some(next) = self.peek(None) {
                    if next == '"' {
                        break;
                    }
                    self.advance(None);
                }
                let Some(raw_string) = self.source.get(current_idx+1..self.idx + 1) else {
                    panic!("Failed to collect string from the source.");
                };
                let string_value = raw_string.iter().collect::<String>();
                self.results.ast.push(Tokens::String(string_value));
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
                        self.results.ast.push(Tokens::Word(Word {
                            which: word,
                            plural: false,
                        }));
                    }
                }
            }
        }
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
    let input_str = std::fs::read_to_string("cbt/test.cbt").unwrap();

    let mut lexer = Lexer {
        source: input_str.chars().collect(),
        results: ast::AST { ast: vec![] },
        idx: 0,
    };

    lexer.lex_all();
    println!(
        "Reconstructed AST:\n\n {:?}\n...Parsing.",
        lexer.results.reconstruct()
    );
    let mut parser = p::Parser {
        source: lexer.results,
        idx: 0,
        nodes: vec![],
        working_on: None,
    };
    parser.parse_all();
    println!("{:#?}", parser.nodes);
}
