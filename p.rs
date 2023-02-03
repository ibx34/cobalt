use std::{env::current_exe, f32::consts::E, iter::Peekable, path::Iter};

use crate::{
    ast::AST,
    node::{Expr, LiteralExpr, Stmt},
    Token, Tokens, Word, Words,
};

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
    pub fn parse_stmt(&mut self) -> Option<Stmt> {
        if let Some(current) = self.source.peek() {
            match &current.inner {
                Tokens::Word(word) => match word.which {
                    Words::Define => {
                        self.advance();
                        if let Some(next) = self.source.peek() {
                            let Tokens::Word(word) = &next.inner else {
                                panic!("Expected the word MODULE but instead got: {next:?}");
                            };
                            match word.which {
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

                                    let Some(block) = self.parse_block() else {
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
                        self.advance();
                    }
                    _ => {}
                },

                _ => {}
            }
        }
        None
    }

    pub fn parse_block(&mut self) -> Option<Stmt> {
        let mut nodes: Vec<Box<Stmt>> = Vec::new();
        while let Some(next) = self.source.peek() {
            if next.inner
                == Tokens::Word(Word {
                    which: Words::End,
                    plural: false,
                })
            {
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

// impl Parser {
//     pub fn parse(&mut self) {
//         if let Some(node) = self.parse_stmt(false) {
//             self.nodes.push(node);
//         }
// let Some(current) = self.current() else {
//     panic!("Failed to get current token.");
// };
// match &current.inner {
//     Tokens::Word(word) => match word.which {
//         Words::Define => {
//             self.advance(None);
//             let Some(current) = self.current() else {
//                 panic!("Failed to get next token.");
//             };
//             match current.inner {
//                 Tokens::Word(Word {
//                     which: Words::Module,
//                     plural: false,
//                 }) => {
//                     self.advance(None);

//                     if let Some(string) = self.parse_string() {
//                         println!("Start module: {:?}", string);
//                         // We can advance, or not and add the the current token
//                         // (in this case the word Module) in the expected list.
//                         self.expect_and_skip(vec![
//                             Tokens::Word(Word {
//                                 which: Words::With,
//                                 plural: false,
//                             }),
//                             Tokens::Word(Word {
//                                 which: Words::Contents,
//                                 plural: false,
//                             }),
//                             Tokens::Colon,
//                         ]);
//                         // Handle block:
//                         let Some(stmts) = self.parse_stmt() else {
//                             panic!("Failed to parse statements");
//                         };

//                         self.expect_and_skip(vec![
//                             Tokens::Word(Word {
//                                 which: Words::The,
//                                 plural: false,
//                             }),
//                             Tokens::Word(Word {
//                                 which: Words::Module,
//                                 plural: false,
//                             }),
//                             Tokens::Period,
//                         ]);

//                         self.nodes.push(Stmt::Module {
//                             name: string,
//                             nodes: stmts
//                                 .into_iter()
//                                 .map(|e| Box::new(e))
//                                 .collect::<Vec<Box<Stmt>>>(),
//                         })
//                     }
//                 }
//                 _ => panic!("Unexpected word following DEFINE word."),
//             }
//         }
//         _ => {}
//     },
//     _ => {}
// }
//     }

//     pub fn parse_expr(&mut self) -> Option<Expr> {
//         let Some(current_token) = self.peek(None) else {
//             return None;
//         };
//         match current_token.inner {
//             Tokens::String => {
//                 let Some(string) = self.parse_string() else {
//                     panic!("Could not parse string");
//                 };
//                 self.advance(None);
//                 return Some(Expr::StringLiteral(string));
//             }
//             _ => panic!("Unsupported token type for expr"),
//         }
//     }

//     pub fn parse_block(&mut self) -> Option<Vec<Stmt>> {
//         println!("Parsing block");
//         let mut nodes = Vec::new();
//         while !self.eof(None) {
//             let stmt = self.parse_stmt(true);
//             println!("{:?}", stmt);
//             if let Some(stmt) = stmt {
//                 println!("BLOCK STMT => {:?}", stmt);
//                 nodes.push(stmt);
//             }
//             self.advance(None);
//         }
//         println!("Finished block parsing");
//         return Some(nodes);
//     }

//     pub fn parse_stmt(&mut self, in_block: bool) -> Option<Stmt> {
//         self.advance(None);
//         if let Some(current) = self.current() {
//             match &current.inner {
//                 Tokens::Word(word) => {
//                     match word.which {
//                         Words::End => {
//                             if !word.plural {
//                                 println!("[IN BLOCK: {in_block:?}] END WORD");
//                                 self.advance(None);
//                                 return None;
//                             }
//                         }
//                         Words::Define => {
//                             if let Some(next) = self.peek(None) {
//                                 let Tokens::Word(word) = &next.inner else {
//                                     // not sure what to do her
//                                     panic!("H");
//                                 };
//                                 if word.which == Words::Module {
//                                     self.advance(None);
//                                     let Some(name) = self.parse_string() else {
//                                         panic!("H");
//                                     };
//                                     println!("[IN BLOCK: {in_block:?}] Module {name:?}");

//                                     self.advance(Some(2));
//                                     let stmt = self.parse_block();

//                                     if let Some(stmt) = stmt {
//                                         println!("[IN BLOCK: {in_block:?}] Got stmt: {stmt:?}");
//                                         // return Some(Stmt::Module {
//                                         //     name,
//                                         //     nodes: vec![Box::new(stmt)],
//                                         // });
//                                     }
//                                 }
//                             }
//                         }
//                         _ => {}
//                     }
//                 }
//                 Tokens::String => {
//                     println!("Got variable");
//                     // Currently variables are the only thing that start with a string
//                     // so its safe to assume this is a variable, if valid.
//                     if let Some(var_name) = self.parse_string() {
//                         println!("Variable {var_name:?}");
//                         self.expect_and_skip(vec![
//                             Tokens::Word(Word {
//                                 which: Words::Is,
//                                 plural: false,
//                             }),
//                             Tokens::Word(Word {
//                                 which: Words::Equal,
//                                 plural: false,
//                             }),
//                             Tokens::Word(Word {
//                                 which: Words::To,
//                                 plural: false,
//                             }),
//                         ]);
//                         let value = self.parse_expr();

//                         return Some(Stmt::Variable {
//                             name: var_name,
//                             value,
//                         });
//                     }
//                 }
//                 _ => {}
//             }
//         }
//         None
//     }

//     pub fn parse_string(&mut self) -> Option<String> {
//         // Ideally this would be a peak. better design TBD
//         let Some(current) = self.current() else {
//             println!("Current is none.");
//             return None;
//         };
//         println!("-> {current:?}");
//         if current.inner == Tokens::String {
//             if let Some(string) = self.source_str.get(current.location.span.clone()) {
//                 let string = string.iter().collect::<String>();
//                 return Some(string);
//             }
//         }
//         None
//     }

//     pub fn expect_and_skip(&mut self, expect: Vec<Tokens>) {
//         for token in expect {
//             if let Some(current_tok) = self.peek(None) {
//                 if current_tok.inner != token {
//                     println!("{:?} != {:?}", current_tok.inner, token);
//                     // self.advance(None);
//                     panic!("Expected smth else...");
//                 }
//             } else {
//                 panic!("Failed to get current token");
//             }
//             self.advance(None);
//         }
//     }

//     pub fn parse_all(&mut self) {
//         while !self.eof(None) {
//             self.parse();
//         }
//     }

//     pub fn current(&self) -> Option<&Token> {
//         self.source.ast.get(self.idx)
//     }
//     pub fn eof(&self, amount: Option<usize>) -> bool {
//         let idx = amount.unwrap_or(self.idx);

//         if self.source.ast.get(idx).is_some() {
//             return false;
//         }
//         true
//     }
//     pub fn advance(&mut self, amount: Option<usize>) -> bool {
//         let amount = amount.unwrap_or(1);

//         if !self.eof(Some(amount)) {
//             self.idx += amount;
//             return true;
//         }
//         false
//     }
//     pub fn peek(&mut self, amount: Option<usize>) -> Option<&Token> {
//         self.source.ast.get(self.idx + amount.unwrap_or(1))
//     }
// }
