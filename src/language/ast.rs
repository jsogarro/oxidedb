use crate::types::atom::Atom;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Atom(Atom),
    Symbol(String),
    BinaryOp {
        left: Box<Expr>,
        operator: BinaryOperator,
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

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    Negate,
}
