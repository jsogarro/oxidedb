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
