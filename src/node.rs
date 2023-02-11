#[derive(Debug, Clone)]
pub enum LiteralExpr {
    String(String),
    Bool(bool)
}

#[derive(Debug, Clone)]
pub struct FunctionCall {
    pub func: Box<Expr>,
    pub args: Option<Vec<Box<Expr>>>,
}

#[derive(Debug, Clone)]
/// This is short hand for a phrase (like `EQUAL TO`)
pub enum BinaryOperators {
    EqualTo,
}

#[derive(Debug, Clone)]
pub struct Binary {
    pub l: Box<Expr>,
    pub r: Box<Expr>,
    pub op: BinaryOperators,
}

#[derive(Debug, Clone)]
pub struct Condition {
    pub then: Box<Stmt>,
    pub el: Option<Box<Stmt>>,
    pub condition: Box<Expr>,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Literal(LiteralExpr),
    Call(FunctionCall),
    BinaryOp(Binary),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VariableType {
    String,
    Bool
}

#[derive(Debug, Clone)]
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
    Condition(Condition),
}
