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
            _ => false,
        }
    }
}

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
