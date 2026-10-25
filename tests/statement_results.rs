//! A statement's value is dropped before the next statement runs, so a discarded `v` does not
//! keep a second reference to the vector that `v[0]:x` then has to copy.

use oxidedb::{Interpreter, Value};
use std::rc::Rc;

fn vector_ptr(i: &Interpreter, name: &str) -> *const () {
    match i.get(name) {
        Some(Value::Vector(c)) => Rc::as_ptr(c) as *const (),
        other => panic!("{name} is not a vector: {other:?}"),
    }
}

#[test]
fn a_discarded_statement_value_does_not_force_a_copy_on_the_next_amend() {
    let mut i = Interpreter::new();
    i.eval_line("v:til 5").unwrap();
    let before = vector_ptr(&i, "v");

    // control: no discarded value in play, the amend is in place
    i.eval_line("v[0]:9").unwrap();
    assert_eq!(vector_ptr(&i, "v"), before);

    i.eval_line("v;v[1]:8").unwrap();
    assert_eq!(vector_ptr(&i, "v"), before, "`v;` kept a reference alive");
    i.eval_line("v;v;v[2]:7;").unwrap();
    assert_eq!(vector_ptr(&i, "v"), before);
}
