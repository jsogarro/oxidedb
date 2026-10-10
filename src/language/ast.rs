use crate::types::value::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// A literal: an atom or a vector. Evaluating it is a reference-count bump.
    Lit(Value),
    Symbol(String),
    BinaryOp {
        left: Box<Expr>,
        operator: Verb,
        right: Box<Expr>,
    },
    UnaryOp {
        operator: UnaryOperator,
        operand: Box<Expr>,
    },
    Assignment {
        name: String,
        value: Box<Expr>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verb {
    Add,
    Subtract,
    Multiply,
    Divide,
    Equal,
    Less,
    Greater,
    NotEqual,
    LessEqual,
    GreaterEqual,
    Take,
    Join,
    Key,
}

impl Verb {
    /// The verb as written in source.
    pub fn symbol(&self) -> &'static str {
        match self {
            Verb::Add => "+",
            Verb::Subtract => "-",
            Verb::Multiply => "*",
            Verb::Divide => "%",
            Verb::Equal => "=",
            Verb::Less => "<",
            Verb::Greater => ">",
            Verb::NotEqual => "<>",
            Verb::LessEqual => "<=",
            Verb::GreaterEqual => ">=",
            Verb::Take => "#",
            Verb::Join => ",",
            Verb::Key => "!",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    Negate,
}
