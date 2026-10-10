use super::atom::Atom;
use super::column::Column;
use crate::error::{QError, QResult};
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

/// The deepest a general list may nest (an atom or a vector is depth 0, a list is one more
/// than its deepest item). Equality, arithmetic, negation, display and drop all recurse once
/// per level. Measured at 64 levels (smallest thread stack that survives, steps of a power of
/// two): debug needs between 128 and 256 KB for equality and arithmetic (negation under 128 KB),
/// release at most 64 KB, display and drop under 4 KB. A 2 MB thread keeps an 8x margin in the
/// worst case. The cap matters because the per-line budget cannot see nesting built
/// up over many lines. EVERY operation that can create nesting (item assignment, list
/// literals, `enlist`, ...) must call [`Value::check_nesting`] on what it is about to put in.
pub const MAX_VALUE_DEPTH: usize = 64;

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

    /// Whether this value nests deeper than `limit` levels. Never recurses past `limit` levels
    /// and visits each shared (`Rc`) list once, so it is safe on any value and costs the number
    /// of distinct lists in it, not the number of paths through them.
    pub fn nests_deeper_than(&self, limit: usize) -> bool {
        fn within(
            v: &Value,
            room: usize,
            seen: &mut HashMap<*const Vec<Value>, usize>,
        ) -> Option<usize> {
            let Value::List(items) = v else {
                return Some(0);
            };
            let key = Rc::as_ptr(items);
            if let Some(&d) = seen.get(&key) {
                return (d <= room).then_some(d);
            }
            let room_below = room.checked_sub(1)?;
            let mut deepest = 0;
            for item in items.iter() {
                deepest = deepest.max(within(item, room_below, seen)?);
            }
            seen.insert(key, deepest + 1);
            Some(deepest + 1)
        }
        within(self, limit, &mut HashMap::new()).is_none()
    }

    /// `'limit` unless this value, placed `levels_above` lists down, stays within
    /// `MAX_VALUE_DEPTH`. Call it on every value about to become an item of a list (with
    /// `levels_above` 1), before changing anything. Cost: the distinct lists inside `self`.
    pub fn check_nesting(&self, levels_above: usize) -> QResult<()> {
        if self.nests_deeper_than(MAX_VALUE_DEPTH.saturating_sub(levels_above)) {
            return Err(QError::Limit(format!(
                "nesting deeper than {MAX_VALUE_DEPTH}"
            )));
        }
        Ok(())
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
            // the same list is equal to itself (nulls equal nulls): no walk through shared nesting
            (Value::List(a), Value::List(b)) => Rc::ptr_eq(a, b) || a == b,
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
