#[derive(Debug, Clone)]
pub enum LiteralExpr {
    String(String),
}

#[derive(Debug, Clone)]
pub enum Expr {
    Literal(LiteralExpr),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VariableType {
    String,
}

#[derive(Debug)]
pub enum Stmt {
    Block(Vec<Box<Stmt>>),
    Module {
        name: LiteralExpr,
        nodes: Box<Stmt>,
    },
    Expr(Expr),
    Variable {
        name: String,
        ty: VariableType,
        value: Option<Expr>,
    },
    Function {
        name: LiteralExpr,
        nodes: Box<Stmt>,
    },
    // Just for now
    Token(crate::Tokens),
}
