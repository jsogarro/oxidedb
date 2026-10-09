use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Atom {
    // Basic numeric types
    Boolean(bool),
    Integer(i64),
    Float(f64),
    Character(char),

    // Temporal types
    Date(NaiveDate),
    Time(NaiveTime),
    Timestamp(DateTime<Utc>),

    // String/Symbol
    Symbol(String),

    // Null values for each type
    NullBoolean,
    NullInteger,
    NullFloat,
    NullCharacter,
    NullDate,
    NullTime,
    NullTimestamp,
    NullSymbol,
}

impl Atom {
    pub fn type_code(&self) -> i8 {
        match self {
            Atom::Boolean(_) | Atom::NullBoolean => -1,
            Atom::Integer(_) | Atom::NullInteger => -7,
            Atom::Float(_) | Atom::NullFloat => -9,
            Atom::Character(_) | Atom::NullCharacter => -10,
            Atom::Date(_) | Atom::NullDate => -14,
            Atom::Time(_) | Atom::NullTime => -19,
            Atom::Timestamp(_) | Atom::NullTimestamp => -12,
            Atom::Symbol(_) | Atom::NullSymbol => -11,
        }
    }

    pub fn is_null(&self) -> bool {
        matches!(
            self,
            Atom::NullBoolean
                | Atom::NullInteger
                | Atom::NullFloat
                | Atom::NullCharacter
                | Atom::NullDate
                | Atom::NullTime
                | Atom::NullTimestamp
                | Atom::NullSymbol
        )
    }

    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            Atom::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_integer(&self) -> Option<i64> {
        match self {
            Atom::Integer(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match self {
            Atom::Float(f) => Some(*f),
            Atom::Integer(i) => Some(*i as f64),
            _ => None,
        }
    }

    pub fn as_character(&self) -> Option<char> {
        match self {
            Atom::Character(c) => Some(*c),
            _ => None,
        }
    }

    pub fn as_symbol(&self) -> Option<&str> {
        match self {
            Atom::Symbol(s) => Some(s),
            _ => None,
        }
    }
}

/// q-style float display with the default 7 significant digits: the value is
/// rounded first, so an integral result gets the `f` suffix (`845f`). Magnitudes
/// >= 1e15 or < 1e-4 use exponent form (`1e+20`, `1e-10`).
fn write_float(f: &mut fmt::Formatter, fl: f64) -> fmt::Result {
    let r: f64 = format!("{:.6e}", fl).parse().unwrap_or(fl);
    let (abs, exp_form) = (r.abs(), r.abs() >= 1e15 || (r != 0.0 && r.abs() < 1e-4));
    if exp_form {
        let s = format!("{:e}", r);
        write!(
            f,
            "{}",
            if s.contains("e-") {
                s
            } else {
                s.replace('e', "e+")
            }
        )
    } else if r.fract() == 0.0 && abs < 1e15 {
        write!(f, "{}f", r)
    } else {
        write!(f, "{}", r)
    }
}

impl fmt::Display for Atom {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Atom::Boolean(b) => write!(f, "{}", if *b { "1b" } else { "0b" }),
            Atom::Integer(i) => write!(f, "{}", i),
            Atom::Float(fl) if fl.is_nan() => write!(f, "0n"),
            Atom::Float(fl) if fl.is_infinite() => {
                write!(f, "{}0w", if *fl < 0.0 { "-" } else { "" })
            }
            Atom::Float(fl) => write_float(f, *fl),
            Atom::Character(c) => write!(f, "\"{}\"", c),
            Atom::Date(d) => write!(f, "{}", d.format("%Y.%m.%d")),
            Atom::Time(t) => write!(f, "{}", t.format("%H:%M:%S.%3f")),
            Atom::Timestamp(ts) => write!(f, "{}", ts.format("%Y.%m.%dT%H:%M:%S.%3fZ")),
            Atom::Symbol(s) => write!(f, "`{}", s),
            Atom::NullBoolean => write!(f, "0Nb"),
            Atom::NullInteger => write!(f, "0N"),
            Atom::NullFloat => write!(f, "0n"),
            Atom::NullCharacter => write!(f, "\" \""),
            Atom::NullDate => write!(f, "0Nd"),
            Atom::NullTime => write!(f, "0Nt"),
            Atom::NullTimestamp => write!(f, "0Np"),
            Atom::NullSymbol => write!(f, "`"),
        }
    }
}

impl From<bool> for Atom {
    fn from(b: bool) -> Self {
        Atom::Boolean(b)
    }
}

impl From<i64> for Atom {
    fn from(i: i64) -> Self {
        Atom::Integer(i)
    }
}

impl From<f64> for Atom {
    fn from(f: f64) -> Self {
        Atom::Float(f)
    }
}

impl From<char> for Atom {
    fn from(c: char) -> Self {
        Atom::Character(c)
    }
}

impl From<String> for Atom {
    fn from(s: String) -> Self {
        Atom::Symbol(s)
    }
}

impl From<&str> for Atom {
    fn from(s: &str) -> Self {
        Atom::Symbol(s.to_string())
    }
}
