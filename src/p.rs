use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};
use std::iter::Peekable;

use crate::cg::var;
use crate::node::{Binary, BinaryOperators, Condition, FunctionCall};
use crate::{
    errors::ErrorClient,
    node::{Expr, LiteralExpr, Stmt, VariableType},
    Token, Tokens, Word, Words,
};

pub enum BlockType {
    Named(String, usize),
    Unamed(usize),
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
            self.nodes.push(stmt);
        }
    }
    pub fn parse_expr(&mut self) -> Option<Expr> {
        if let Some(current) = self.source.peek() {
            match current.inner {
                Tokens::String => {
                    let Some(lit) = self.parse_string() else {
                        panic!("Failed to parse function name");
                    };
                    self.advance();
                    return Some(lit);
                }
                _ => {
                    panic!("Unsupported.")
                }
            }
        }
        None
    }
    pub fn parse_stmt(&mut self) -> Option<Stmt> {
        if let Some(current) = self.source.peek() {
            match &current.inner {
                Tokens::Word(word) => match word.which {
                    Words::If => {
                        self.advance();

                        let Some(left) = self.parse_expr() else {panic!("failed")};
                        let op = if self.expect(Tokens::Word(Word {
                            which: Words::Is,
                            plural: false,
                        })) {
                            let (r1, r2) = (
                                self.expect(Tokens::Word(Word {
                                    which: Words::Equal,
                                    plural: false,
                                })),
                                self.expect(Tokens::Word(Word {
                                    which: Words::To,
                                    plural: false,
                                })),
                            );

                            if r1 && r2 {
                                BinaryOperators::EqualTo
                            } else {
                                panic!("Unsupported operator");
                            }
                        } else {
                            panic!("Unsupported operator");
                        };
                        let Some(right) = self.parse_expr() else {panic!("failed")};

                        self.expect_and_skip(vec![
                            Tokens::Word(Word {
                                which: Words::Then,
                                plural: false,
                            }),
                            Tokens::Word(Word {
                                which: Words::Do,
                                plural: false,
                            }),
                        ]);

                        let Some(then) = self.parse_block(BlockType::Unamed(0)) else {
                            panic!("Failed to parse if block");
                        };
                        return Some(Stmt::Condition(Condition {
                            then: Box::new(then),
                            el: None,
                            condition: Box::new(Expr::BinaryOp(Binary {
                                l: Box::new(left),
                                r: Box::new(right),
                                op,
                            })),
                        }));
                    }
                    Words::Call => {
                        self.advance();
                        if let Some(next) = self.source.peek() {
                            let Tokens::Word(word) = &next.inner else {
                                panic!("Expected the word MODULE but instead got: {next:?}");
                            };
                            match word.which {
                                Words::Function => {
                                    self.advance();
                                    let Some(Expr::Literal(lit)) = self.parse_string() else {
                                        panic!("Failed to parse function name");
                                    };
                                    self.advance();

                                    if self.expect(Tokens::Word(Word {
                                        which: Words::With,
                                        plural: false,
                                    })) {
                                        if self.expect(Tokens::Word(Word {
                                            which: Words::The,
                                            plural: false,
                                        })) {
                                            self.expect_and_skip(vec![Tokens::Word(Word {
                                                which: Words::Argument,
                                                plural: false,
                                            })]);

                                            // TODO: make a parse_expr function to make stuff like this WAY easier.
                                            let Some(Expr::Literal(only_arg)) = self.parse_string() else {
                                                panic!("Failed to parse function name");
                                            };
                                            self.advance();

                                            return Some(Stmt::Expr(Expr::Call(FunctionCall {
                                                func: Box::new(Expr::Literal(lit)),
                                                args: Some(vec![Box::new(Expr::Literal(only_arg))]),
                                            })));
                                        } else {
                                            unimplemented!(
                                                "Multiple function call args is NOT supported."
                                            );
                                        }
                                    } else {
                                        self.advance();
                                        return Some(Stmt::Expr(Expr::Call(FunctionCall {
                                            func: Box::new(Expr::Literal(lit)),
                                            args: None,
                                        })));
                                    }
                                }
                                _ => unimplemented!(),
                            }
                        }
                    }
                    Words::Define => {
                        self.advance();
                        if let Some(next) = self.source.peek() {
                            let Tokens::Word(word) = &next.inner else {
                                panic!("Expected the word MODULE but instead got: {next:?}");
                            };
                            match word.which {
                                Words::Function => {
                                    self.advance();
                                    let Some(Expr::Literal(LiteralExpr::String(func_name))) = self.parse_string() else {
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
                                    // todo: change this to handle the other stuff
                                    if self.expect(Tokens::Word(Word {
                                        which: Words::That,
                                        plural: false,
                                    })) {
                                        self.expect_and_skip(vec![
                                            Tokens::Word(Word {
                                                which: Words::Returns,
                                                plural: false,
                                            }),
                                            Tokens::Word(Word {
                                                which: Words::A,
                                                plural: false,
                                            }),
                                        ]);

                                        if !self.expect(Tokens::Colon) {
                                            panic!("Expected a colon");
                                        }

                                        let Some(function_body) = self
                                            .parse_block(BlockType::Named(func_name.clone(), 1))
                                            else {
                                                panic!("Failed to get function body.");
                                            };
                                        return Some(Stmt::Function {
                                            name: LiteralExpr::String(func_name),
                                            nodes: Box::new(function_body),
                                        });
                                    }
                                }
                                Words::Module => {
                                    self.advance();
                                    let Some(Expr::Literal(LiteralExpr::String(module_name))) = self.parse_string() else {
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
                        let Some(Expr::Literal(LiteralExpr::String(variable_name))) = self.parse_string() else {
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
                        let Some(expr) = self.parse_string() else {
                            panic!("Failed to parse module name");
                        };
                        self.advance();
                        if self.expect_and_return(Tokens::Period).is_none() {
                            panic!("END WITH A PERIOD DAMNIT.")
                        }
                        return Some(Stmt::Variable {
                            name: variable_name,
                            value: Some(expr),
                            ty: VariableType::String,
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
                        let Some(Expr::Literal(LiteralExpr::String(name2))) = self.parse_string() else {
                            panic!("Expected module name");
                        };
                        if name2 != name {
                            panic!("Expected module name {name:?} but instead got {name2:?}");
                        }
                        self.advance();
                        if self.expect_and_return(Tokens::Period).is_none() {
                            panic!("END WITH A PERIOD DAMNIT.")
                        }
                        self.advance();
                    }
                    BlockType::Unamed(version) => {
                        if version == 0 {
                            self.expect_and_skip(vec![Tokens::Word(Word {
                                which: Words::If,
                                plural: false,
                            })]);
                            self.advance();
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
                    let mut error = ErrorClient::new("0001", crate::errors::MessageKind::ERROR);
                    error.end_process(true);
                    // TODO: The parser should have a context field that is aware of this kind of stuff.
                    error.set_file(
                        "module_level_func.cbt",
                        "./tests/pass/module_level_func.cbt",
                    );
                    error.set_span(current_tok.location.span.clone());
                    let note = format!(
                        "Expected the word `{}` but instead got `{}`",
                        token.to_string(),
                        current_tok.inner.to_string()
                    );
                    error.add_label(Some(&note));
                    error.build_and_emit();
                }
            } else {
                panic!("Failed to get current token");
            }
            self.advance();
        }
    }

    pub fn expect(&mut self, expect: Tokens) -> bool {
        if let Some(next) = self.source.peek() {
            if next.inner == expect {
                self.advance();
                return true;
            }
        };
        return false;
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

    pub fn parse_string(&mut self) -> Option<Expr> {
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
            return Some(Expr::Literal(LiteralExpr::String(string)));
        }
        None
    }

    pub fn advance(&mut self) -> Option<Token> {
        self.idx += 1;
        self.source.next()
    }
}
