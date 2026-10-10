use oxidedb::Atom;

fn show(f: f64) -> String {
    format!("{}", Atom::Float(f))
}

#[test]
fn float_display_keeps_decimal() {
    assert_eq!(show(20.0), "20f");
    assert_eq!(show(3.14158), "3.14158");
    assert_eq!(show(-2.5), "-2.5");
    assert_eq!(show(-0.0), "-0f");
}

#[test]
fn display_float_specials() {
    assert_eq!(show(f64::INFINITY), "0w");
    assert_eq!(show(f64::NEG_INFINITY), "-0w");
    assert_eq!(show(f64::NAN), "0n");
}

#[test]
fn display_large_floats_use_exponent() {
    assert_eq!(show(1e20), "1e+20");
    assert_eq!(show(-1e20), "-1e+20");
}

#[test]
fn display_float_seven_significant_digits() {
    assert_eq!(show(6.081081081081082), "6.081081");
    assert_eq!(show(844.9999999999999), "845f");
    assert_eq!(show(1.0 / 3.0), "0.3333333");
    assert_eq!(show(0.1 + 0.2), "0.3");
    assert_eq!(show(123456.789), "123456.8");
    assert_eq!(show(3.14158), "3.14158");
    assert_eq!(show(-2.5), "-2.5");
    assert_eq!(show(1e20), "1e+20");
    assert_eq!(show(1e-10), "1e-10");
    assert_eq!(show(f64::INFINITY), "0w");
    assert_eq!(show(f64::NAN), "0n");
}

#[test]
fn display_float_g_format_thresholds() {
    assert_eq!(show(9999999.0), "9999999f");
    assert_eq!(show(10000000.0), "1e+07");
    assert_eq!(show(12345678.0), "1.234568e+07");
    assert_eq!(show(123456789012.0), "1.234568e+11");
    assert_eq!(show(99999999.5), "1e+08");
    assert_eq!(show(0.0001), "0.0001");
    assert_eq!(show(0.00001), "1e-05");
    assert_eq!(show(1e-10), "1e-10");
    assert_eq!(show(1e20), "1e+20");
    assert_eq!(show(-0.0), "-0f");
}

use oxidedb::types::column::Column;
use oxidedb::types::sym::Sym;

fn col(c: Column) -> String {
    format!("{}", c)
}

#[test]
fn display_col_long() {
    assert_eq!(col(Column::Long(vec![1, 2, 3])), "1 2 3");
    assert_eq!(col(Column::Long(vec![5])), ",5");
    assert_eq!(col(Column::Long(vec![1, i64::MIN, 3])), "1 0N 3");
    assert_eq!(col(Column::Long(vec![i64::MIN])), ",0N");
    assert_eq!(col(Column::Long(vec![-1, -20])), "-1 -20");
}

#[test]
fn display_col_float() {
    assert_eq!(col(Column::Float(vec![1.0, 2.5, 3.0])), "1 2.5 3");
    assert_eq!(col(Column::Float(vec![1.0, 2.0, 3.0])), "1 2 3f");
    assert_eq!(col(Column::Float(vec![20.0])), ",20f");
    assert_eq!(col(Column::Float(vec![2.5])), ",2.5");
    assert_eq!(col(Column::Float(vec![1.0, f64::NAN, 3.0])), "1 0n 3");
    assert_eq!(col(Column::Float(vec![1.0, f64::INFINITY])), "1 0w");
}

#[test]
fn display_col_float_specials() {
    assert_eq!(col(Column::Float(vec![f64::NAN, 1.5])), "0n 1.5");
    assert_eq!(col(Column::Float(vec![f64::NEG_INFINITY, 1.5])), "-0w 1.5");
    assert_eq!(col(Column::Float(vec![f64::NAN, f64::NAN])), "0n 0n");
    assert_eq!(
        col(Column::Float(vec![f64::INFINITY, f64::INFINITY])),
        "0w 0w"
    );
    assert_eq!(col(Column::Float(vec![f64::NAN])), ",0n");
    assert_eq!(
        col(Column::Float(vec![1.0, 2.0, 3.0, 4.0, 5.0])),
        "1 2 3 4 5f"
    );
    assert_eq!(col(Column::Float(vec![1e20, 1.0])), "1e+20 1");
    assert_eq!(col(Column::Float(vec![0.1 + 0.2, 1.0])), "0.3 1");
}

#[test]
fn display_col_bool() {
    assert_eq!(col(Column::Bool(vec![true, false, true])), "101b");
    assert_eq!(col(Column::Bool(vec![true])), ",1b");
    assert_eq!(col(Column::Bool(vec![false])), ",0b");
}

#[test]
fn display_col_sym() {
    let s = |x: &str| Sym::intern(x);
    assert_eq!(col(Column::Sym(vec![s("a"), s("b")])), "`a`b");
    assert_eq!(col(Column::Sym(vec![s("a")])), ",`a");
    assert_eq!(col(Column::Sym(vec![s("a"), Sym::NULL, s("b")])), "`a``b");
}

#[test]
fn display_col_char() {
    assert_eq!(col(Column::Char(vec!['a', 'b', 'c'])), "\"abc\"");
    assert_eq!(col(Column::Char(vec!['a'])), ",\"a\"");
    assert_eq!(col(Column::Char(vec![])), "\"\"");
    assert_eq!(col(Column::Char(vec!['"'])), ",\"\\\"\"");
    assert_eq!(col(Column::Char(vec!['\\', 'a'])), "\"\\\\a\"");
    assert_eq!(
        col(Column::Char(vec!['a', '\n', '\t', '\r'])),
        "\"a\\n\\t\\r\""
    );
}

#[test]
fn display_col_empty() {
    assert_eq!(col(Column::Long(vec![])), "`long$()");
    assert_eq!(col(Column::Float(vec![])), "`float$()");
    assert_eq!(col(Column::Bool(vec![])), "`boolean$()");
    assert_eq!(col(Column::Sym(vec![])), "`symbol$()");
}

#[test]
fn display_col_float_nan_equality_is_derived() {
    // Documents the derived PartialEq: a float column holding 0n is not equal to itself.
    // To be decided with the Value type.
    let c = Column::Float(vec![1.0, f64::NAN]);
    assert_ne!(c, c.clone());
}

#[test]
fn display_atom_char_escapes_quote() {
    assert_eq!(format!("{}", Atom::Character('"')), "\"\\\"\"");
    assert_eq!(format!("{}", Atom::Character('a')), "\"a\"");
}
