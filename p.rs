use crate::{ast::AST, node::Node, Tokens, Words};

pub struct Parser {
    pub source: AST,
    pub idx: usize,
    pub nodes: Vec<Node>,
    pub working_on: Option<Node>,
}

impl Parser {
    pub fn parse(&mut self) {
        let Some(current) = self.current() else {
            panic!("Failed to get current token.");
        };
        match current {
            Tokens::Word(word) => match word.which {
                Words::Define => {
                    let Some(Tokens::Word(next_word)) = self.peek(None) else {
                        panic!("Failed to peek the next token");
                    };
                    let actual_next_word = &next_word.which;
                    match *actual_next_word {
                        Words::Module => {
                            self.advance(None);
                            let Some(Tokens::String(module_name)) = self.peek(None) else {
                                panic!("Failed to peek the next token OR next token was not a string.");
                            };
                            self.working_on = Some(Node::Module {
                                name: module_name.to_owned(),
                                nodes: vec![],
                            });
                            self.advance(None);
                        }
                        Words::Function => {}
                        _ => panic!(
                            "The word \"{:?}\" cannot follow the word \"{:?}\"",
                            actual_next_word, word.which
                        ),
                    }
                }
                Words::End => {
                    let Some(Tokens::Word(next_word)) = self.peek(None) else {
                        panic!("Failed to peek the next token");
                    };
                    let actual_next_word = &next_word.which;
                    if *actual_next_word != Words::Module && *actual_next_word != Words::Function {
                        panic!(
                            "The word \"{:?}\" cannot follow the word \"{:?}\"",
                            actual_next_word, word.which
                        );
                    }
                    self.advance(None);
                    let Some(Tokens::Period) = self.peek(None) else {
                        panic!("The next token MUST be a period.");
                    };
                    self.advance(None);
                    let Some(working_on) = self.working_on.to_owned() else {
                        panic!("Attemping to end a block failed.");
                    };
                    self.nodes.push(working_on);
                    self.working_on = None;
                }
                _ => {}
            },
            _ => {}
        }
    }

    pub fn parse_all(&mut self) {
        while !self.eof(None) {
            self.parse();
            self.advance(None);
        }
    }

    pub fn current(&self) -> Option<&Tokens> {
        self.source.ast.get(self.idx)
    }
    pub fn eof(&self, amount: Option<usize>) -> bool {
        let idx = amount.unwrap_or(self.idx);

        if self.source.ast.get(idx).is_some() {
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
    pub fn peek(&self, amount: Option<usize>) -> Option<&Tokens> {
        self.source.ast.get(self.idx + amount.unwrap_or(1))
    }
}
