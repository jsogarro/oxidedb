//! Runs in its own process, so nothing has been interned before: the table is `` ` `` (index 0)
//! and the first new symbol must get index 1.
use oxidedb::types::sym::Sym;

#[test]
fn first_interned_symbol_gets_index_one() {
    assert_eq!(format!("{:?}", Sym::NULL), "Sym(0)");
    let first = Sym::intern("first_symbol");
    assert_eq!(format!("{first:?}"), "Sym(1)");
    assert_eq!(format!("{:?}", Sym::intern("second_symbol")), "Sym(2)");
    assert_eq!(Sym::intern("first_symbol"), first);
}
