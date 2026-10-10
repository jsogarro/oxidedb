//! q-style errors: a quote followed by a short name (`'type`), with a detail
//! where the name alone would lose information.

/// An error raised while lexing, parsing or evaluating O code.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum QError {
    #[error("'type")]
    Type,
    #[error("'length")]
    Length,
    #[error("'rank")]
    Rank,
    #[error("'index")]
    Index,
    #[error("'domain")]
    Domain,
    #[error("'nyi: {0}")]
    Nyi(String),
    #[error("'parse: {0}")]
    Parse(String),
    #[error("'{0} (Undefined variable)")]
    Undefined(String),
    #[error("'stack")]
    Stack,
    #[error("'overflow")]
    Overflow,
    #[error("'{0}")]
    Signal(String),
}
