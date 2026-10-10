use chrono::{TimeZone, Utc};
use oxidedb::types::atom::Atom;
use oxidedb::types::sym::Sym;

#[test]
fn atoms_sym_intern_identity() {
    assert_eq!(Sym::intern("a"), Sym::intern("a"));
    assert_ne!(Sym::intern("a"), Sym::intern("b"));
    assert_eq!(Sym::intern("hello").as_str(), "hello");
    assert_eq!(Sym::intern(""), Sym::NULL);
    assert_eq!(Sym::NULL.as_str(), "");
    assert_eq!(Sym::NULL.to_string(), "`");
    assert_eq!(Sym::intern("abc").to_string(), "`abc");
    assert_eq!(Atom::from("x"), Atom::Symbol(Sym::intern("x")));
    assert_eq!(Atom::from("x").as_symbol(), Some("x"));
}

#[test]
fn atoms_null_sentinels() {
    let long_null = Atom::Integer(i64::MIN);
    assert!(long_null.is_null());
    assert_eq!(long_null.to_string(), "0N");
    let float_null = Atom::Float(f64::NAN);
    assert!(float_null.is_null());
    assert_eq!(float_null.to_string(), "0n");
    assert!(Atom::Character(' ').is_null());
    assert!(Atom::Symbol(Sym::NULL).is_null());
    assert_eq!(Atom::Symbol(Sym::NULL).to_string(), "`");
    assert!(Atom::NullDate.is_null());
    assert!(!Atom::Integer(0).is_null());
    assert!(!Atom::Float(0.0).is_null());
    assert!(!Atom::Character('a').is_null());
    assert!(!Atom::Symbol(Sym::intern("a")).is_null());
    assert!(!Atom::Boolean(false).is_null());
}

#[test]
fn atoms_type_codes_unchanged() {
    assert_eq!(Atom::Boolean(true).type_code(), -1);
    assert_eq!(Atom::Integer(1).type_code(), -7);
    assert_eq!(Atom::Integer(i64::MIN).type_code(), -7);
    assert_eq!(Atom::Float(1.0).type_code(), -9);
    assert_eq!(Atom::Float(f64::NAN).type_code(), -9);
    assert_eq!(Atom::Character('a').type_code(), -10);
    assert_eq!(Atom::Symbol(Sym::NULL).type_code(), -11);
    assert_eq!(Atom::Symbol(Sym::intern("a")).type_code(), -11);
}

#[test]
fn atoms_timestamp_display_q_format() {
    let ts = Utc.with_ymd_and_hms(2024, 1, 15, 9, 30, 0).unwrap()
        + chrono::Duration::nanoseconds(123_456_789);
    assert_eq!(
        Atom::Timestamp(ts).to_string(),
        "2024.01.15D09:30:00.123456789"
    );
}

#[test]
fn atoms_temporal_nulls_and_values() {
    use chrono::{NaiveDate, NaiveTime};
    assert!(Atom::NullDate.is_null());
    assert!(Atom::NullTime.is_null());
    assert!(Atom::NullTimestamp.is_null());
    let d = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
    assert!(!Atom::Date(d).is_null());
    assert!(!Atom::Time(NaiveTime::from_hms_opt(9, 30, 0).unwrap()).is_null());
    assert!(!Atom::Timestamp(Utc.with_ymd_and_hms(2024, 1, 15, 9, 30, 0).unwrap()).is_null());
}

#[test]
fn atoms_non_null_edge_values() {
    assert!(!Atom::Integer(i64::MAX).is_null());
    assert!(!Atom::Integer(0).is_null());
    assert!(!Atom::Float(f64::INFINITY).is_null());
    assert!(!Atom::Float(f64::NEG_INFINITY).is_null());
    assert!(!Atom::Float(0.0).is_null());
    assert!(!Atom::Character('\0').is_null());
    assert!(!Atom::Character('a').is_null());
}

#[test]
fn atoms_float_null_equals_null() {
    // Nulls compare equal (decided with the Value type); 0f and -0f stay equal.
    assert_eq!(Atom::Float(f64::NAN), Atom::Float(f64::NAN));
    assert_ne!(Atom::Float(f64::NAN), Atom::Float(1.0));
    assert_eq!(Atom::Float(0.0), Atom::Float(-0.0));
}
