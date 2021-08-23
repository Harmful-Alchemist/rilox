use crate::token::Token;
use crate::literal::Literal;

pub trait Expr {
    fn pretty_print(&self) -> String;
}

pub struct Binary<'a> {
    pub(crate) left: &'a dyn Expr,
    pub(crate) operator: Token,
    pub(crate) right: &'a dyn Expr,
}

impl Expr for Binary<'_> {
    fn pretty_print(&self) -> String {
        format!("({} {} {})", self.operator.lexeme, self.left.pretty_print(), self.right.pretty_print())
    }
}

pub struct Grouping<'a> {
    pub(crate) expression: &'a dyn Expr,
}

impl Expr for Grouping<'_> {
    fn pretty_print(&self) -> String {
        format!("(group {})", self.expression.pretty_print())
    }
}

pub struct LiteralExpr {
    pub(crate) value: crate::literal::Literal,
}

impl Expr for LiteralExpr {
    fn pretty_print(&self) -> String {
        match &self.value {
            Literal::String(a) => a.clone(),
            Literal::Number(a) => format!("{}", a),
            Literal::None => String::from("nil")
        }
    }
}

pub struct Unary<'a> {
    pub(crate) operator: Token,
    pub(crate) right: &'a dyn Expr,
}

impl Expr for Unary<'_> {
    fn pretty_print(&self) -> String {
        format!("({} {})", self.operator.lexeme, self.right.pretty_print())
    }
}