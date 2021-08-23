use crate::literal::Literal;
use crate::token::Token;

pub trait Expr {
    fn pretty_print(&self) -> String;
}

pub struct Binary {
    pub(crate) left: Box<dyn Expr>,
    pub(crate) operator: Token,
    pub(crate) right: Box<dyn Expr>,
}

impl Expr for Binary {
    fn pretty_print(&self) -> String {
        format!(
            "({} {} {})",
            self.operator.lexeme,
            self.left.pretty_print(),
            self.right.pretty_print()
        )
    }
}

pub struct Grouping {
    pub(crate) expression: Box<dyn Expr>,
}

impl Expr for Grouping {
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
            Literal::String(a) => format!("\"{}\"", a.clone()),
            Literal::Number(a) => format!("{}", a),
            Literal::Bool(a) => format!("{}", a),
            Literal::None => String::from("nil"),
        }
    }
}

pub struct Unary {
    pub(crate) operator: Token,
    pub(crate) right: Box<dyn Expr>,
}

impl Expr for Unary {
    fn pretty_print(&self) -> String {
        format!("({} {})", self.operator.lexeme, self.right.pretty_print())
    }
}
