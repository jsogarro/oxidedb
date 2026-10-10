use super::atom::Atom;
use super::column::Column;
use std::fmt;
use std::rc::Rc;

/// Any O value: an atom, a typed vector, or a general (mixed or nested) list.
/// Vectors and lists are reference counted, so cloning a variable is cheap.
#[derive(Debug, Clone)]
pub enum Value {
    Atom(Atom),
    Vector(Rc<Column>),
    List(Rc<Vec<Value>>),
}

impl Value {
    /// q type code: negative for atoms, positive for vectors, 0 for a general list.
    pub fn type_code(&self) -> i8 {
        match self {
            Value::Atom(a) => a.type_code(),
            Value::Vector(c) => c.type_code(),
            Value::List(_) => 0,
        }
    }

    /// The one place lists are built: same-typed atoms collapse into a vector
    /// (no int to float promotion); empty or anything else is a general list.
    pub fn from_items(items: Vec<Value>) -> Value {
        let atoms: Option<Vec<Atom>> = items
            .iter()
            .map(|v| match v {
                Value::Atom(a) => Some(a.clone()),
                _ => None,
            })
            .collect();
        match atoms.as_deref().and_then(Column::from_atoms) {
            Some(col) => Value::Vector(Rc::new(col)),
            None => Value::List(Rc::new(items)),
        }
    }
}

/// Nulls compare equal; see `Atom` and `Column`.
impl PartialEq for Value {
    fn eq(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Atom(a), Value::Atom(b)) => a == b,
            (Value::Vector(a), Value::Vector(b)) => a == b,
            (Value::List(a), Value::List(b)) => a == b,
            (Value::Atom(_) | Value::Vector(_) | Value::List(_), _) => false,
        }
    }
}

impl Eq for Value {}

impl PartialEq<Atom> for Value {
    fn eq(&self, other: &Atom) -> bool {
        matches!(self, Value::Atom(a) if a == other)
    }
}

impl PartialEq<Value> for Atom {
    fn eq(&self, other: &Value) -> bool {
        other == self
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Atom(a) => a.fmt(f),
            Value::Vector(c) => c.fmt(f),
            // ponytail: one item per line; q's exact nested layout arrives with #51.
            Value::List(items) if items.is_empty() => f.write_str("()"),
            Value::List(items) => {
                for (i, v) in items.iter().enumerate() {
                    if i > 0 {
                        f.write_str("\n")?;
                    }
                    v.fmt(f)?;
                }
                Ok(())
            }
        }
    }
}
