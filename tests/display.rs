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
