use super::stmts::Stmt;

#[derive(Debug)]
pub enum Expr<'a> {
    Literal(ExprLit<'a>),
    Assignment(AssignmentExpr<'a>),
    BinaryOp(BinaryExpr<'a>),
    Condition(ConditionExpr<'a>),
}

#[derive(Debug)]
pub struct ConditionExpr<'a> {
    pub then: Box<Stmt<'a>>,
    pub el: Option<Box<Stmt<'a>>>,
    pub condition: Box<Expr<'a>>,
}

#[derive(Debug)]
pub struct AssignmentExpr<'a> {
    pub op: BinaryOperators,
    pub l: Box<Expr<'a>>,
    pub r: Box<Expr<'a>>,
}

#[derive(Debug)]
pub enum ExprLit<'a> {
    String(&'a str),
    Identifier(&'a str),
    Boolean(bool),
    Number(f64),
}

#[derive(Debug, Clone)]
/// This is short hand for a phrase (like `EQUAL TO`)
pub enum BinaryOperators {
    EqualTo,
}

#[derive(Debug)]
pub struct BinaryExpr<'a> {
    pub op: BinaryOperators,
    pub l: Box<Expr<'a>>,
    pub r: Box<Expr<'a>>,
}
