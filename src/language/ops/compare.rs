//! `= <> < <= > >=` over atoms, vectors and general lists, atomic like the
//! arithmetic verbs. Results are booleans (a `Bool` column for vectors).
//! Numbers (bool, long, float) compare by value; null equals null and sorts
//! lowest; symbols and characters compare among themselves.

use super::{atomic, Flat};
use crate::error::{QError, QResult};
use crate::language::ast::Verb;
use crate::types::atom::Atom;
use crate::types::column::Column;
use crate::types::sym::Sym;
use crate::types::value::Value;
use std::cmp::Ordering;
use std::rc::Rc;

/// One comparable element. Longs stay exact; a float on either side makes both floats.
#[derive(Clone, Copy)]
enum Item {
    Long(i64),
    Float(f64),
    Char(char),
    Sym(Sym),
}

/// Only elements of the same kind compare (bool, long and float are all numbers).
#[derive(PartialEq)]
enum Kind {
    Number,
    Char,
    Sym,
}

fn item(a: &Atom) -> QResult<Item> {
    match a {
        Atom::Boolean(x) => Ok(Item::Long(i64::from(*x))),
        Atom::Integer(x) => Ok(Item::Long(*x)),
        Atom::Float(x) => Ok(Item::Float(*x)),
        Atom::Character(x) => Ok(Item::Char(*x)),
        Atom::Symbol(x) => Ok(Item::Sym(*x)),
        _ => Err(QError::Type),
    }
}

fn kind(f: &Flat) -> QResult<Kind> {
    Ok(match f {
        Flat::Atom(a) => match item(a)? {
            Item::Long(_) | Item::Float(_) => Kind::Number,
            Item::Char(_) => Kind::Char,
            Item::Sym(_) => Kind::Sym,
        },
        Flat::Col(Column::Bool(_) | Column::Long(_) | Column::Float(_)) => Kind::Number,
        Flat::Col(Column::Char(_)) => Kind::Char,
        Flat::Col(Column::Sym(_)) => Kind::Sym,
    })
}

/// Element `i` of an operand; an atom is the same element at every `i`.
fn at(f: &Flat, i: usize) -> QResult<Item> {
    match f {
        Flat::Atom(a) => item(a),
        Flat::Col(Column::Bool(v)) => Ok(Item::Long(i64::from(v[i]))),
        Flat::Col(Column::Long(v)) => Ok(Item::Long(v[i])),
        Flat::Col(Column::Float(v)) => Ok(Item::Float(v[i])),
        Flat::Col(Column::Char(v)) => Ok(Item::Char(v[i])),
        Flat::Col(Column::Sym(v)) => Ok(Item::Sym(v[i])),
    }
}

/// Null sorts below everything, including -0w, and equals itself.
fn float_order(a: f64, b: f64) -> Ordering {
    a.partial_cmp(&b)
        .unwrap_or_else(|| b.is_nan().cmp(&a.is_nan()))
}

/// A long null is the NaN null once a float is involved.
fn as_float(n: i64) -> f64 {
    if n == i64::MIN {
        f64::NAN
    } else {
        n as f64
    }
}

/// Order two elements of the same kind (the caller has checked the kinds).
fn order(a: Item, b: Item) -> Ordering {
    match (a, b) {
        (Item::Long(x), Item::Long(y)) => x.cmp(&y),
        (Item::Float(x), Item::Float(y)) => float_order(x, y),
        (Item::Long(x), Item::Float(y)) => float_order(as_float(x), y),
        (Item::Float(x), Item::Long(y)) => float_order(x, as_float(y)),
        (Item::Char(x), Item::Char(y)) => x.cmp(&y),
        // Symbols order by name; the null symbol is the empty name.
        (Item::Sym(x), Item::Sym(y)) => x.as_str().cmp(y.as_str()),
        _ => Ordering::Equal,
    }
}

fn holds(verb: Verb, o: Ordering) -> bool {
    match verb {
        Verb::Equal => o == Ordering::Equal,
        Verb::NotEqual => o != Ordering::Equal,
        Verb::Less => o == Ordering::Less,
        Verb::LessEqual => o != Ordering::Greater,
        Verb::Greater => o == Ordering::Greater,
        _ => o != Ordering::Less,
    }
}

pub fn dyad(verb: Verb, left: &Value, right: &Value) -> QResult<Value> {
    atomic(verb, left, right, flat)
}

fn len(f: &Flat) -> Option<usize> {
    match f {
        Flat::Atom(_) => None,
        Flat::Col(c) => Some(c.len()),
    }
}

fn flat(verb: Verb, left: Flat, right: Flat) -> QResult<Value> {
    if kind(&left)? != kind(&right)? {
        return Err(QError::Type);
    }
    let cmp = |i| Ok(holds(verb, order(at(&left, i)?, at(&right, i)?)));
    let n = match (len(&left), len(&right)) {
        (None, None) => return cmp(0).map(|t| Value::Atom(Atom::Boolean(t))),
        (Some(n), None) | (None, Some(n)) => n,
        (Some(n), Some(m)) if n == m => n,
        (Some(_), Some(_)) => return Err(QError::Length),
    };
    let out = (0..n).map(cmp).collect::<QResult<_>>()?;
    Ok(Value::Vector(Rc::new(Column::Bool(out))))
}
