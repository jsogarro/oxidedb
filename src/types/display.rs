use super::atom::{float_digits, Atom};
use super::column::Column;
use std::fmt::{self, Write};

fn join<T>(
    f: &mut fmt::Formatter,
    items: &[T],
    mut w: impl FnMut(&mut fmt::Formatter, &T) -> fmt::Result,
) -> fmt::Result {
    for (i, x) in items.iter().enumerate() {
        if i > 0 {
            f.write_char(' ')?;
        }
        w(f, x)?;
    }
    Ok(())
}

impl fmt::Display for Column {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_empty() {
            return match self {
                Column::Char(_) => f.write_str("\"\""),
                Column::Bool(_) => f.write_str("`boolean$()"),
                Column::Long(_) => f.write_str("`long$()"),
                Column::Float(_) => f.write_str("`float$()"),
                Column::Sym(_) => f.write_str("`symbol$()"),
            };
        }
        if self.len() == 1 {
            f.write_char(',')?;
        }
        match self {
            Column::Bool(v) => {
                for &b in v {
                    f.write_char(if b { '1' } else { '0' })?;
                }
                f.write_char('b')
            }
            Column::Long(v) => join(f, v, |f, &x| write!(f, "{}", Atom::Integer(x))),
            Column::Float(v) => {
                // Some(integral) for finite elements; None for 0n / 0w, which are neutral.
                let parts: Vec<(String, Option<bool>)> = v
                    .iter()
                    .map(|&x| match x {
                        x if x.is_nan() => ("0n".into(), None),
                        x if x.is_infinite() => ((if x < 0.0 { "-0w" } else { "0w" }).into(), None),
                        x => {
                            let (d, integral) = float_digits(x);
                            (d, Some(integral))
                        }
                    })
                    .collect();
                // One trailing `f` when no element needs a decimal point or exponent.
                let suffix = parts.iter().any(|p| p.1 == Some(true))
                    && parts.iter().all(|p| p.1 != Some(false));
                join(f, &parts, |f, p| f.write_str(&p.0))?;
                if suffix {
                    f.write_char('f')?;
                }
                Ok(())
            }
            Column::Char(v) => {
                f.write_char('"')?;
                for &c in v {
                    match c {
                        '"' => f.write_str("\\\"")?,
                        '\\' => f.write_str("\\\\")?,
                        '\n' => f.write_str("\\n")?,
                        '\t' => f.write_str("\\t")?,
                        '\r' => f.write_str("\\r")?,
                        c => f.write_char(c)?,
                    }
                }
                f.write_char('"')
            }
            Column::Sym(v) => v.iter().try_for_each(|s| write!(f, "{}", s)),
        }
    }
}
