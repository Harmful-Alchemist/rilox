#[derive(Debug, Clone)]
pub(crate) enum Literal {
    String(String),
    Number(f64),
    Bool(bool),
    None,
}
