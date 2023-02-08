#[derive(Debug, Clone)]
pub enum LiteralExpr {
    String(String),
}

#[derive(Debug, Clone)]
pub struct FunctionCall {
    pub func: Box<Expr>,
    pub args: Option<Vec<Box<Expr>>>,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Literal(LiteralExpr),
    Call(FunctionCall),
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
