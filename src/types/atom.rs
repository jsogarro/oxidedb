use super::display::write_escaped;
use super::sym::Sym;
use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use std::fmt;

#[derive(Debug, Clone)]
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
    Symbol(Sym),

    // Typed temporal nulls. Long/float/char/symbol nulls are sentinels:
    // `Integer(i64::MIN)`, `Float(NaN)`, `Character(' ')`, `Symbol(Sym::NULL)`.
    NullDate,
    NullTime,
    NullTimestamp,
}

/// Float equality where null equals null: `NaN == NaN`, while `0f == -0f`.
pub(crate) fn float_eq(a: f64, b: f64) -> bool {
    a == b || (a.is_nan() && b.is_nan())
}

/// Nulls compare equal (`0n` equals `0n`), unlike derived IEEE equality.
impl PartialEq for Atom {
    fn eq(&self, other: &Atom) -> bool {
        use Atom::*;
        match (self, other) {
            (Boolean(a), Boolean(b)) => a == b,
            (Integer(a), Integer(b)) => a == b,
            (Float(a), Float(b)) => float_eq(*a, *b),
            (Character(a), Character(b)) => a == b,
            (Date(a), Date(b)) => a == b,
            (Time(a), Time(b)) => a == b,
            (Timestamp(a), Timestamp(b)) => a == b,
            (Symbol(a), Symbol(b)) => a == b,
            (NullDate, NullDate) | (NullTime, NullTime) | (NullTimestamp, NullTimestamp) => true,
            // Exhaustive on the left operand, so a new variant must be decided here.
            (
                Boolean(_) | Integer(_) | Float(_) | Character(_) | Date(_) | Time(_)
                | Timestamp(_) | Symbol(_) | NullDate | NullTime | NullTimestamp,
                _,
            ) => false,
        }
    }
}

impl Eq for Atom {}

impl Atom {
    pub fn type_code(&self) -> i8 {
        match self {
            Atom::Boolean(_) => -1,
            Atom::Integer(_) => -7,
            Atom::Float(_) => -9,
            Atom::Character(_) => -10,
            Atom::Date(_) | Atom::NullDate => -14,
            Atom::Time(_) | Atom::NullTime => -19,
            Atom::Timestamp(_) | Atom::NullTimestamp => -12,
            Atom::Symbol(_) => -11,
        }
    }

    pub fn is_null(&self) -> bool {
        match self {
            Atom::Integer(i) => *i == i64::MIN,
            Atom::Float(f) => f.is_nan(),
            Atom::Character(c) => *c == ' ',
            Atom::Symbol(s) => *s == Sym::NULL,
            Atom::NullDate | Atom::NullTime | Atom::NullTimestamp => true,
            Atom::Boolean(_) | Atom::Date(_) | Atom::Time(_) | Atom::Timestamp(_) => false,
        }
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
            Atom::Symbol(s) => Some(s.as_str()),
            _ => None,
        }
    }
}

/// q-style float digits, equivalent to C `%.7g` (q's default `\P 7`): exponent
/// form when the decimal exponent is < -4 or >= 7 (`1e-05`, `1.234568e+07`),
/// otherwise fixed with trailing zeros trimmed. The flag is true when the
/// result is integral and so takes the `f` suffix as an atom (`845f`).
pub(crate) fn float_digits(fl: f64) -> (String, bool) {
    let sci = format!("{:.6e}", fl);
    let (mantissa, exp) = sci.split_once('e').unwrap_or((&sci, "0"));
    let exp: i32 = exp.parse().unwrap_or(0);
    if !(-4..7).contains(&exp) {
        let mantissa = mantissa.trim_end_matches('0').trim_end_matches('.');
        let sign = if exp < 0 { '-' } else { '+' };
        return (format!("{}e{}{:02}", mantissa, sign, exp.abs()), false);
    }
    let r: f64 = sci.parse().unwrap_or(fl);
    (r.to_string(), r.fract() == 0.0)
}

fn write_float(f: &mut fmt::Formatter, fl: f64) -> fmt::Result {
    let (digits, integral) = float_digits(fl);
    write!(f, "{}{}", digits, if integral { "f" } else { "" })
}

impl fmt::Display for Atom {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Atom::Boolean(b) => write!(f, "{}", if *b { "1b" } else { "0b" }),
            Atom::Integer(i64::MIN) => write!(f, "0N"),
            Atom::Integer(i) => write!(f, "{}", i),
            Atom::Float(fl) if fl.is_nan() => write!(f, "0n"),
            Atom::Float(fl) if fl.is_infinite() => {
                write!(f, "{}0w", if *fl < 0.0 { "-" } else { "" })
            }
            Atom::Float(fl) => write_float(f, *fl),
            Atom::Character(c) => {
                f.write_str("\"")?;
                write_escaped(f, *c)?;
                f.write_str("\"")
            }
            Atom::Date(d) => write!(f, "{}", d.format("%Y.%m.%d")),
            Atom::Time(t) => write!(f, "{}", t.format("%H:%M:%S.%3f")),
            Atom::Timestamp(ts) => write!(f, "{}", ts.format("%Y.%m.%dD%H:%M:%S.%9f")),
            Atom::Symbol(s) => write!(f, "{}", s),
            Atom::NullDate => write!(f, "0Nd"),
            Atom::NullTime => write!(f, "0Nt"),
            Atom::NullTimestamp => write!(f, "0Np"),
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
        Atom::Symbol(Sym::intern(&s))
    }
}

impl From<&str> for Atom {
    fn from(s: &str) -> Self {
        Atom::Symbol(Sym::intern(s))
    }
}
