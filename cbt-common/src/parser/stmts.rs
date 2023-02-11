use super::expr::Expr;

#[derive(Debug)]
pub enum Stmt<'a> {
    Expr(Expr<'a>),
    Block(Vec<Box<Stmt<'a>>>),
    FunctionDef(FunctionDef<'a>),
}

#[derive(Debug)]
pub enum TypeDec {
    Literal(LiteralType),
}

#[derive(Debug)]
pub enum LiteralType {
    Boolean,
    Number,
    Ident,
}

#[derive(Debug)]
pub struct FunctionArgument<'a> {
    pub name: &'a str,
    pub ty: TypeDec,
}

#[derive(Debug)]
pub struct FunctionDef<'a> {
    pub name: &'a str,
    pub ret_ty: Option<TypeDec>,
    pub args: Option<Vec<FunctionArgument<'a>>>,
    pub body: Box<Stmt<'a>>,
}
