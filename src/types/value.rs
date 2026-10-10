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

/// The most items a general list may hold counting shared sub-lists once per path
/// ([`Value::logical_size`]). Operations on a list walk every path, so this bounds their time
/// and memory however the list was built. Measured on a list at 10,000,000 (the cap for flat
/// vectors, `MAX_ELEMS`): `l+1`, `neg l`, `l=l`, `l,l` and printing took 6.9 s in release and
/// 37.8 s in debug and wrote 77 MB; at one tenth of that each is a second or so, hence this
/// lower figure.
pub const MAX_LOGICAL_ITEMS: usize = 1_000_000;

/// Logical sizes, memoised per list so a value shared along many paths is measured once.
#[derive(Default)]
pub struct Sizer {
    seen: HashMap<*const Vec<Value>, u64>,
}

impl Sizer {
    /// `v`'s logical size; `u64::MAX` when it nests deeper than `room` levels.
    pub fn size(&mut self, v: &Value, room: usize) -> u64 {
        match v {
            Value::Atom(_) => 1,
            Value::Vector(c) => (c.len() as u64).max(1),
            Value::List(items) => {
                let key = Rc::as_ptr(items);
                if let Some(&n) = self.seen.get(&key) {
                    return n;
                }
                let Some(below) = room.checked_sub(1) else {
                    return u64::MAX;
                };
                let mut n = 1u64;
                for x in items.iter() {
                    n = n.saturating_add(self.size(x, below));
                    if n == u64::MAX {
                        // saturated, or cut short by the depth bound: stop at once, so a
                        // too-deep shared value is not walked path by path. Not memoised.
                        return n;
                    }
                }
                self.seen.insert(key, n);
                n
            }
        }
    }
}

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

    /// The number of items in this value counting a shared sub-list once per path through it:
    /// an atom is 1, a vector its length (at least 1), a list 1 plus its items. This is what
    /// display, arithmetic, comparison and `neg` walk, however much sharing hides it.
    /// Saturating, and linear in the distinct lists (memoised by `Rc` pointer); a value
    /// nested deeper than `MAX_VALUE_DEPTH` is `u64::MAX` without being followed.
    pub fn logical_size(&self) -> u64 {
        Sizer::default().size(self, MAX_VALUE_DEPTH)
    }

    /// THE guard for anything that makes or stores a general list. `'limit` unless this value,
    /// placed `levels_above` lists down, (1) nests at most `MAX_VALUE_DEPTH` levels and (2) has
    /// a [`logical_size`](Value::logical_size) of at most `MAX_LOGICAL_ITEMS`. Both limits matter:
    /// depth bounds the stack, size bounds time and memory, and sharing (`l[0]:l; l[1]:l`
    /// repeated) doubles the size every round while the depth stays small. Call it on every
    /// value about to become an item of a list (with `levels_above` 1) and on every list a verb
    /// returns (with 0), before changing anything. Cost: the distinct lists inside `self`.
    pub fn check_nesting(&self, levels_above: usize) -> QResult<()> {
        self.check_nesting_within(levels_above, MAX_VALUE_DEPTH, MAX_LOGICAL_ITEMS)
    }

    /// [`check_nesting`](Value::check_nesting) with the two caps given, so tests can sit
    /// exactly on a boundary without building ten million items.
    pub fn check_nesting_within(
        &self,
        levels_above: usize,
        max_depth: usize,
        max_items: usize,
    ) -> QResult<()> {
        if self.nests_deeper_than(max_depth.saturating_sub(levels_above)) {
            return Err(QError::Limit(format!("nesting deeper than {max_depth}")));
        }
        if self.logical_size() > max_items as u64 {
            return Err(QError::Limit(format!(
                "value larger than {max_items} items"
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
            Value::List(items) => super::display::fmt_list(f, items),
        }
    }
}
