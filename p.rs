use std::iter::Peekable;

use crate::{
    node::{Expr, LiteralExpr, Stmt},
    Token, Tokens, Word, Words,
};

pub enum BlockType {
    Named(String, usize),
}

pub struct Parser<T>
where
    T: Iterator<Item = Token>,
{
    pub source: Peekable<T>,
    pub source_str: Vec<char>,
    pub idx: usize,
    pub nodes: Vec<Stmt>,
}

impl<'a, T> Parser<T>
where
    T: Iterator<Item = Token>,
{
    pub fn parse(&mut self) {
        while let Some(stmt) = self.parse_stmt() {
            println!("stmt");
            self.nodes.push(stmt);
        }
    }
    pub fn parse_stmt(&mut self) -> Option<Stmt> {
        if let Some(current) = self.source.peek() {
            match &current.inner {
                Tokens::DollarSign => return Some(Stmt::Token(Tokens::DollarSign)),
                Tokens::Word(word) => match word.which {
                    Words::Define => {
                        self.advance();
                        if let Some(next) = self.source.peek() {
                            let Tokens::Word(word) = &next.inner else {
                                panic!("Expected the word MODULE but instead got: {next:?}");
                            };
                            match word.which {
                                Words::Function => {
                                    println!("GOT FUNCTION");
                                    self.advance();
                                    let Some(Stmt::Expr(Expr::Literal(LiteralExpr::String(func_name)))) = self.parse_string() else {
                                        panic!("Failed to parse function name");
                                    };
                                    self.advance();

                                    /*
                                        Here we can diverge. The function can have:
                                        - No arguments
                                        - One Argumet
                                        - Or many arguments

                                        Depending on which of the previous is used will determine the syntax required,
                                        We can start by checking for the word "THAT". If it is "THAT" then we know the function takes no arguments.
                                    */
                                    if self
                                        .expect_and_return(Tokens::Word(Word {
                                            which: Words::That,
                                            plural: false,
                                        }))
                                        .is_some()
                                    {
                                        // why?
                                        self.advance();
                                        self.advance();
                                        if self.expect_and_return(Tokens::Colon).is_none() {
                                            panic!("Expected a colon.");
                                        }
                                        self.advance();

                                        let function_body = self
                                            .parse_block(BlockType::Named(func_name.clone(), 1))
                                            .unwrap();
                                        return Some(Stmt::Function {
                                            name: LiteralExpr::String(func_name),
                                            nodes: Box::new(function_body),
                                        });
                                    }
                                }
                                Words::Module => {
                                    self.advance();
                                    let Some(Stmt::Expr(Expr::Literal(LiteralExpr::String(module_name)))) = self.parse_string() else {
                                        panic!("Failed to parse module name");
                                    };
                                    self.advance();
                                    self.expect_and_skip(vec![
                                        Tokens::Word(Word {
                                            which: Words::With,
                                            plural: false,
                                        }),
                                        Tokens::Word(Word {
                                            which: Words::Contents,
                                            plural: false,
                                        }),
                                        Tokens::Colon,
                                    ]);

                                    let Some(block) = self.parse_block(BlockType::Named(module_name.clone(), 0)) else {
                                        panic!("Failed to parse module content");
                                    };
                                    return Some(Stmt::Module {
                                        name: LiteralExpr::String(module_name),
                                        nodes: Box::new(block),
                                    });
                                }
                                _ => {}
                            }
                        }
                    }
                    Words::Set => {
                        self.advance();
                        let Some(Stmt::Expr(Expr::Literal(LiteralExpr::String(variable_name)))) = self.parse_string() else {
                            panic!("Failed to parse module name");
                        };
                        self.advance();
                        self.expect_and_skip(vec![
                            Tokens::Word(Word {
                                which: Words::Equal,
                                plural: false,
                            }),
                            Tokens::Word(Word {
                                which: Words::To,
                                plural: false,
                            }),
                        ]);
                        let Some(Stmt::Expr(expr)) = self.parse_string() else {
                            panic!("Failed to parse module name");
                        };
                        self.advance();
                        if self.expect_and_return(Tokens::Period).is_none() {
                            panic!("END WITH A PERIOD DAMNIT.")
                        }
                        return Some(Stmt::Variable {
                            name: variable_name,
                            value: Some(expr),
                        });
                    }
                    _ => {}
                },

                _ => {}
            }
        }
        None
    }

    pub fn parse_block(&mut self, block_type: BlockType) -> Option<Stmt> {
        let mut nodes: Vec<Box<Stmt>> = Vec::new();
        while let Some(next) = self.source.peek() {
            if next.inner
                == Tokens::Word(Word {
                    which: Words::End,
                    plural: false,
                })
            {
                self.advance();
                match block_type {
                    BlockType::Named(name, version) => {
                        let word = match version {
                            0 => Word {
                                which: Words::Module,
                                plural: false,
                            },
                            1 => Word {
                                which: Words::Function,
                                plural: false,
                            },
                            _ => panic!("Don't know this type of named block."),
                        };
                        if self.expect_and_return(Tokens::Word(word)).is_none() {
                            panic!("Expected word");
                        };
                        self.advance();
                        let Some(Stmt::Expr(Expr::Literal(LiteralExpr::String(name2)))) = self.parse_string() else {
                            panic!("Expected module name");
                        };
                        if name2 != name {
                            panic!("Expected module name {name:?} but instead got {name2:?}");
                        }
                        self.advance();
                        if self.expect_and_return(Tokens::Period).is_none() {
                            panic!("END WITH A PERIOD DAMNIT.")
                        }
                    }
                }
                break;
            }

            if let Some(stmt) = self.parse_stmt() {
                nodes.push(Box::new(stmt))
            }

            self.advance();
        }
        return Some(Stmt::Block(nodes));
    }

    pub fn expect_and_skip(&mut self, expect: Vec<Tokens>) {
        for token in expect {
            if let Some(current_tok) = self.source.peek() {
                if current_tok.inner != token {
                    println!("{:?} != {:?}", current_tok.inner, token);
                    // self.advance(None);
                    panic!("Expected smth else...");
                }
            } else {
                panic!("Failed to get current token");
            }
            _ = self.advance();
        }
    }

    pub fn expect_and_return(&mut self, expect: Tokens) -> Option<&Token> {
        let Some(next) = self.source.peek() else {
            return None;
        };
        if next.inner == expect {
            return Some(next);
        }
        None
    }

    pub fn parse_string(&mut self) -> Option<Stmt> {
        if let Some(current) = self.source.peek() {
            if current.inner != Tokens::String {
                println!("{current:?} IS NOT A STRING");
                return None;
            }
            let Some(string) = self.source_str.get(current.location.span.clone()) else {
                println!("Non existant location");
                return None;
            };
            let string = string.iter().collect::<String>();
            return Some(Stmt::Expr(Expr::Literal(LiteralExpr::String(string))));
        }
        None
    }

    pub fn advance(&mut self) -> Option<Token> {
        self.idx += 1;
        self.source.next()
    }
}
