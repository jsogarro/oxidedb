use super::atom::{float_digits, Atom};
use super::column::Column;
use super::value::Value;
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
                // q adds one trailing `f` only when every element prints as a bare integer.
                let parts: Vec<(String, bool)> = v
                    .iter()
                    .map(|&x| match x {
                        x if x.is_nan() => ("0n".into(), false),
                        x if x.is_infinite() => {
                            ((if x < 0.0 { "-0w" } else { "0w" }).into(), false)
                        }
                        x => float_digits(x),
                    })
                    .collect();
                join(f, &parts, |f, p| f.write_str(&p.0))?;
                if parts.iter().all(|p| p.1) {
                    f.write_char('f')?;
                }
                Ok(())
            }
            Column::Char(v) => {
                f.write_char('"')?;
                v.iter().try_for_each(|&c| write_escaped(f, c))?;
                f.write_char('"')
            }
            Column::Sym(v) => v.iter().try_for_each(|s| write!(f, "{}", s)),
        }
    }
}

/// A char as it appears inside a q string: `"` and `\` backslashed, `\n \r \t` named, other
/// control characters (C0, DEL and C1) and the invisible U+00A0 and U+00AD as three-digit octal `\ooo`, which the lexer reads back.
pub(crate) fn write_escaped(f: &mut fmt::Formatter, c: char) -> fmt::Result {
    match c {
        '"' => f.write_str("\\\""),
        '\\' => f.write_str("\\\\"),
        '\n' => f.write_str("\\n"),
        '\t' => f.write_str("\\t"),
        '\r' => f.write_str("\\r"),
        c if c.is_control() || matches!(c, '\u{a0}' | '\u{ad}') => {
            write!(f, "\\{:03o}", u32::from(c))
        }
        c => f.write_char(c),
    }
}

/// A value as `-3!` shows it: a general list is `()`, `,x` or `(x;y;...)`. Written straight
/// into `w`, so a value near the size cap is not first built as one string.
fn write_repr(w: &mut impl Write, v: &Value) -> fmt::Result {
    match v {
        Value::List(items) => match items.as_slice() {
            [] => w.write_str("()"),
            [one] => {
                w.write_char(',')?;
                write_repr(w, one)
            }
            many => {
                w.write_char('(')?;
                for (i, x) in many.iter().enumerate() {
                    if i > 0 {
                        w.write_char(';')?;
                    }
                    write_repr(w, x)?;
                }
                w.write_char(')')
            }
        },
        other => write!(w, "{other}"),
    }
}

fn repr(v: &Value) -> String {
    let mut s = String::new();
    let _ = write_repr(&mut s, v);
    s
}

/// A char escaped as inside a string.
struct Escaped(char);

impl fmt::Display for Escaped {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write_escaped(f, self.0)
    }
}

/// One element of a vector as q's `string` shows it: no type marks, `""` for a null.
fn cell(c: &Column, i: usize) -> String {
    match c.get(i) {
        Atom::Boolean(b) => u8::from(b).to_string(),
        Atom::Float(x) if x.is_nan() => String::new(),
        Atom::Float(x) if x.is_finite() => float_digits(x).0,
        Atom::Character(ch) => Escaped(ch).to_string(),
        Atom::Symbol(s) => s.as_str().to_string(),
        a if a.is_null() => String::new(),
        a => a.to_string(),
    }
}

/// The cells of one matrix row: a vector's elements, or what `lines` makes of a nested list.
fn cells(row: &Value) -> Vec<String> {
    match row {
        Value::Vector(c) => (0..c.len()).map(|i| cell(c, i)).collect(),
        Value::List(items) => lines(items),
        Value::Atom(_) => unreachable!("an atom item is never a matrix row"),
    }
}

fn item_len(v: &Value) -> usize {
    match v {
        Value::Atom(_) => 1,
        Value::Vector(c) => c.len(),
        Value::List(l) => l.len(),
    }
}

/// Items that are atoms, of different lengths, or all boolean (or all character) vectors
/// keep their own form; otherwise they are the rows of a matrix.
fn own_form(items: &[Value]) -> bool {
    let first = items.first().map(Value::type_code);
    items.iter().any(|v| matches!(v, Value::Atom(_)))
        || items.iter().any(|v| item_len(v) != item_len(&items[0]))
        || (matches!(first, Some(1 | 10)) && items.iter().all(|v| Some(v.type_code()) == first))
}

/// One string per item of a general list, the way the q console builds them; a matrix has its
/// columns padded to the widest cell.
fn lines(items: &[Value]) -> Vec<String> {
    if items.is_empty() || own_form(items) {
        return items.iter().map(repr).collect();
    }
    let rows: Vec<Vec<String>> = items.iter().map(cells).collect();
    let widths: Vec<usize> = (0..rows[0].len())
        .map(|j| rows.iter().map(|r| r[j].chars().count()).max().unwrap_or(0))
        .collect();
    rows.iter()
        .map(|row| {
            // padded by hand: `{:<w$}` panics for widths past 65,535
            let padded: Vec<String> = row
                .iter()
                .zip(&widths)
                .map(|(c, w)| format!("{c}{}", " ".repeat(w - c.chars().count())))
                .collect();
            padded.join(" ")
        })
        .collect()
}

/// A general list as the console shows it: one line per item (nothing at all for `()`).
/// Lines of items in their own form are streamed; only a matrix is built in memory first
/// (its columns must be measured), so its worst case is the whole text at once.
pub(crate) fn fmt_list(f: &mut fmt::Formatter, items: &[Value]) -> fmt::Result {
    if !own_form(items) {
        return f.write_str(&lines(items).join("\n"));
    }
    for (i, x) in items.iter().enumerate() {
        if i > 0 {
            f.write_char('\n')?;
        }
        write_repr(f, x)?;
    }
    Ok(())
}
