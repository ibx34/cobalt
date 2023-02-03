#[derive(Debug, Clone)]
pub enum LiteralExpr {
    String(String),
}

#[derive(Debug, Clone)]
pub enum Expr {
    Literal(LiteralExpr),
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Block(Vec<Box<Stmt>>),
    Module { name: LiteralExpr, nodes: Box<Stmt> },
    Expr(Expr),
    Variable { name: String, value: Option<Expr> },
}
