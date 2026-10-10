use super::atom::{float_eq, Atom};
use super::sym::Sym;

/// A typed vector. Mixed data is never a `Column`.
#[derive(Debug, Clone)]
pub enum Column {
    Bool(Vec<bool>),
    Long(Vec<i64>),
    Float(Vec<f64>),
    Char(Vec<char>),
    Sym(Vec<Sym>),
}

/// Element-wise equality; a float `0n` equals `0n`.
impl PartialEq for Column {
    fn eq(&self, other: &Column) -> bool {
        match (self, other) {
            (Column::Bool(a), Column::Bool(b)) => a == b,
            (Column::Long(a), Column::Long(b)) => a == b,
            (Column::Float(a), Column::Float(b)) => {
                a.len() == b.len() && a.iter().zip(b).all(|(x, y)| float_eq(*x, *y))
            }
            (Column::Char(a), Column::Char(b)) => a == b,
            (Column::Sym(a), Column::Sym(b)) => a == b,
            (
                Column::Bool(_)
                | Column::Long(_)
                | Column::Float(_)
                | Column::Char(_)
                | Column::Sym(_),
                _,
            ) => false,
        }
    }
}

impl Eq for Column {}

impl Column {
    /// q list type code (positive).
    pub fn type_code(&self) -> i8 {
        match self {
            Column::Bool(_) => 1,
            Column::Long(_) => 7,
            Column::Float(_) => 9,
            Column::Char(_) => 10,
            Column::Sym(_) => 11,
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Column::Bool(v) => v.len(),
            Column::Long(v) => v.len(),
            Column::Float(v) => v.len(),
            Column::Char(v) => v.len(),
            Column::Sym(v) => v.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The null of this column's element type. q has no boolean null, so
    /// `Bool` yields `0b`.
    pub fn null_atom(&self) -> Atom {
        match self {
            Column::Bool(_) => Atom::Boolean(false),
            Column::Long(_) => Atom::Integer(i64::MIN),
            Column::Float(_) => Atom::Float(f64::NAN),
            Column::Char(_) => Atom::Character(' '),
            Column::Sym(_) => Atom::Symbol(Sym::NULL),
        }
    }

    /// Element `i` as an atom; out of range gives the typed null.
    pub fn get(&self, i: usize) -> Atom {
        match self {
            Column::Bool(v) => v.get(i).map(|&x| Atom::Boolean(x)),
            Column::Long(v) => v.get(i).map(|&x| Atom::Integer(x)),
            Column::Float(v) => v.get(i).map(|&x| Atom::Float(x)),
            Column::Char(v) => v.get(i).map(|&x| Atom::Character(x)),
            Column::Sym(v) => v.get(i).map(|&x| Atom::Symbol(x)),
        }
        .unwrap_or_else(|| self.null_atom())
    }

    /// Typed gather: element `idx[k]` for each `k`. Negative and out-of-range
    /// indices give the typed null; an empty `idx` gives an empty column of
    /// the same type.
    pub fn index(&self, idx: &[i64]) -> Column {
        fn gather<T: Copy>(v: &[T], idx: &[i64], null: T) -> Vec<T> {
            idx.iter()
                .map(|&i| {
                    usize::try_from(i)
                        .ok()
                        .and_then(|i| v.get(i))
                        .copied()
                        .unwrap_or(null)
                })
                .collect()
        }
        match self {
            Column::Bool(v) => Column::Bool(gather(v, idx, false)),
            Column::Long(v) => Column::Long(gather(v, idx, i64::MIN)),
            Column::Float(v) => Column::Float(gather(v, idx, f64::NAN)),
            Column::Char(v) => Column::Char(gather(v, idx, ' ')),
            Column::Sym(v) => Column::Sym(gather(v, idx, Sym::NULL)),
        }
    }

    /// q `#`: the first `n` elements, wrapping cyclically past the end; a
    /// negative `n` takes `-n` elements ending at the last one, also wrapping.
    /// Taking from an empty column gives `|n|` typed nulls.
    // ponytail: the result length is |n|; callers must bound `n` (the language slice owns the limit).
    pub fn take(&self, n: i64) -> Column {
        let len = self.len();
        let count = n.unsigned_abs() as usize;
        if len == 0 {
            return self.index(&vec![-1; count]);
        }
        let start = if n < 0 { (len - count % len) % len } else { 0 };
        let idx: Vec<i64> = (0..count).map(|k| ((start + k) % len) as i64).collect();
        self.index(&idx)
    }

    /// Join two columns of the same type; `None` when the types differ so the
    /// caller can fall back to a general list.
    pub fn concat(&self, other: &Column) -> Option<Column> {
        fn join<T: Clone>(a: &[T], b: &[T]) -> Vec<T> {
            [a, b].concat()
        }
        Some(match (self, other) {
            (Column::Bool(a), Column::Bool(b)) => Column::Bool(join(a, b)),
            (Column::Long(a), Column::Long(b)) => Column::Long(join(a, b)),
            (Column::Float(a), Column::Float(b)) => Column::Float(join(a, b)),
            (Column::Char(a), Column::Char(b)) => Column::Char(join(a, b)),
            (Column::Sym(a), Column::Sym(b)) => Column::Sym(join(a, b)),
            _ => return None,
        })
    }

    /// Collapse same-typed atoms into a column. `None` for an empty slice (no
    /// type to infer), mixed types (no int to float promotion; that is the
    /// literal parser's job) and atom types without a column (temporals).
    pub fn from_atoms(atoms: &[Atom]) -> Option<Column> {
        fn all<T>(atoms: &[Atom], f: impl Fn(&Atom) -> Option<T>) -> Option<Vec<T>> {
            atoms.iter().map(f).collect()
        }
        Some(match atoms.first()? {
            Atom::Boolean(_) => Column::Bool(all(atoms, |a| match a {
                Atom::Boolean(b) => Some(*b),
                _ => None,
            })?),
            Atom::Integer(_) => Column::Long(all(atoms, |a| match a {
                Atom::Integer(i) => Some(*i),
                _ => None,
            })?),
            Atom::Float(_) => Column::Float(all(atoms, |a| match a {
                Atom::Float(x) => Some(*x),
                _ => None,
            })?),
            Atom::Character(_) => Column::Char(all(atoms, |a| match a {
                Atom::Character(c) => Some(*c),
                _ => None,
            })?),
            Atom::Symbol(_) => Column::Sym(all(atoms, |a| match a {
                Atom::Symbol(s) => Some(*s),
                _ => None,
            })?),
            _ => return None,
        })
    }
}
