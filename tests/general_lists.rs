//! General lists: `(a;b;c)`, `()`, `enlist`, their console display and indexing.
//! Every expected value and every line of display below was produced by q 4.1
//! (`show`), including its column padding for lists of lists.

use oxidedb::{Atom, Interpreter, QError, Value};
use proptest::prelude::*;

fn run(src: &str) -> Result<Option<Value>, QError> {
    Interpreter::new().eval_line(src)
}

fn eval(src: &str) -> Value {
    run(src).unwrap().unwrap()
}

fn shown(src: &str) -> String {
    eval(src).to_string()
}

fn items(v: &Value) -> Vec<Value> {
    match v {
        Value::List(l) => l.to_vec(),
        other => panic!("not a general list: {other:?}"),
    }
}

#[test]
fn glist_collapses_to_vector() {
    for (list, vector) in [
        ("(1;2;3)", "1 2 3"),
        ("(1.5;2.5)", "1.5 2.5"),
        ("(`a;`b;`c)", "`a`b`c"),
        ("(\"a\";\"b\")", "\"ab\""),
        ("(1b;0b;1b)", "101b"),
        ("(1;2)", "1 2"),
        ("(0N;1)", "0N 1"),
    ] {
        assert_eq!(eval(list), eval(vector), "{list}");
        assert!(matches!(eval(list), Value::Vector(_)), "{list}");
    }
    assert_eq!(eval("(1;2;3)").type_code(), 7);
    // Items that are expressions collapse too, evaluated first.
    assert_eq!(eval("(1+1;2*3)"), eval("2 6"));
}

#[test]
fn glist_mixed() {
    for src in [
        "(1;`a;2.5)",
        "(1;2.5)", // no int to float promotion, as in q
        "(1;1b)",
        "(\"a\";1)",
        "(`a;\"b\")",
    ] {
        let v = eval(src);
        assert!(matches!(v, Value::List(_)), "{src}");
        assert_eq!(v.type_code(), 0, "{src}");
    }
    let v = eval("(1;`a;2.5)");
    assert_eq!(items(&v)[0], Atom::Integer(1));
    assert_eq!(items(&v)[2], Atom::Float(2.5));
}

#[test]
fn glist_nested() {
    let v = eval("(1 2;3 4)");
    let rows = items(&v);
    assert_eq!(rows, vec![eval("1 2"), eval("3 4")]);
    assert_eq!(v.type_code(), 0);
    // An inner list that normalises to a vector is a vector item.
    assert_eq!(items(&eval("(1;(2;3))")), vec![eval("1"), eval("2 3")]);
    // One that does not stays a list.
    let inner = eval("(1;(2;`a);3)");
    assert!(matches!(items(&inner)[1], Value::List(_)));
    assert_eq!(eval("((1;2);(3;4))"), eval("(1 2;3 4)"));
    assert_eq!(eval("(1;(2;(3;(4;`a))))").type_code(), 0);
}

#[test]
fn glist_empty() {
    let v = eval("()");
    assert!(matches!(&v, Value::List(l) if l.is_empty()));
    assert_eq!(v.type_code(), 0);
    assert_eq!(eval("count ()"), Atom::Integer(0));
    // `()` is not a typed empty vector.
    assert_ne!(v, eval("0#1 2"));
    assert_eq!(eval("(())").type_code(), 0);
    assert_eq!(eval("(();())"), eval("2#()"));
}

#[test]
fn glist_display_empty_prints_nothing() {
    // q prints nothing at all for `()`: no text, not even an empty line.
    assert_eq!(shown("()"), "");
    let mut repl = oxidedb::repl::Repl::new();
    assert_eq!(repl.eval_line("()").unwrap(), None);
    // ... while a list holding an empty list is one (blank) line, as in q.
    assert_eq!(repl.eval_line("enlist ()").unwrap(), Some(String::new()));
    assert_eq!(repl.eval_line("(();())").unwrap(), Some("\n".to_string()));
    // an empty string is not an empty list
    assert_eq!(repl.eval_line("\"\"").unwrap(), Some("\"\"".to_string()));
}

const DISPLAY: &[(&str, &str)] = &[
    ("(1;2;3)", "1 2 3"),
    ("(1;`a;2.5)", "1\n`a\n2.5"),
    ("(1 2;3 4)", "1 2\n3 4"),
    ("(10b;\"ab\")", "1 0\na b"),
    ("(1e10 1;1 2)", "1e+10 1\n1     2"),
    ("(\"abc\";1)", "\"abc\"\n1"),
    ("(1;(2;3))", "1\n2 3"),
    ("(1;\"abc\";`a)", "1\n\"abc\"\n`a"),
    ("(1;(2;`a))", "1\n(2;`a)"),
    ("(1;2 3;(4;5 6))", "1\n2 3\n(4;5 6)"),
    ("(1;(2;(3;4)))", "1\n(2;3 4)"),
    ("(1;(2;(3;(4;5))))", "1\n(2;(3;4 5))"),
    ("(1;(2 3;4))", "1\n(2 3;4)"),
    ("(1 2;(3;`a))", "1 2 \n3 `a"),
    ("(1 2;(3;\"ab\"))", "1 2   \n3 \"ab\""),
    ("(1 2;3 4 5)", "1 2\n3 4 5"),
    ("(10 2;3 4)", "10 2\n3  4"),
    ("(100 2;3 4;5 6)", "100 2\n3   4\n5   6"),
    ("(1.5 2;3 4)", "1.5 2\n3   4"),
    ("(2.5 1;3 400)", "2.5 1  \n3   400"),
    ("(0N 2;3 4)", "  2\n3 4"),
    ("(1 2;`a`b)", "1 2\na b"),
    ("(`ab`c;`d`ef)", "ab c \nd  ef"),
    ("(`a`b;`c`d)", "a b\nc d"),
    ("(\"ab\";\"cd\")", "\"ab\"\n\"cd\""),
    ("(101b;010b)", "101b\n010b"),
    ("(1 2;\"ab\")", "1 2\na b"),
    ("(1 2;10b)", "1 2\n1 0"),
    ("(1 2;101b)", "1 2\n101b"),
    ("(1 2;`a)", "1 2\n`a"),
    ("(1 2;3)", "1 2\n3"),
    ("(();1)", "()\n1"),
    ("(1;())", "1\n()"),
    ("(();(1;`a))", "()\n(1;`a)"),
    ("(();())", "\n"),
    ("enlist ()", ""),
    ("enlist 1 2", "1 2"),
    ("enlist enlist 1", "1"),
    ("enlist \"abc\"", "\"abc\""),
    ("enlist (1;`a)", "1 `a"),
    ("(enlist 1;2)", ",1\n2"),
    ("(1 2;enlist 3)", "1 2\n,3"),
    ("((1;`a);(2;`b))", "1 `a\n2 `b"),
    ("((1;`a);(2;`b;3))", "(1;`a)\n(2;`b;3)"),
    ("((1;2);3 4)", "1 2\n3 4"),
    ("((1 2;3 4);(5 6;7 8))", "1 2 3 4\n5 6 7 8"),
    ("(((1;`a);2);((3;4);5))", "(1;`a) 2\n3 4    5"),
    ("(1 2 3;(1;2 3;4))", "1 2   3\n1 2 3 4"),
    ("((1;`a);1 2)", "1 `a\n1 2 "),
    ("(1 2;(`a;1.5))", "1  2  \n`a 1.5"),
    ("(1 2;`a`)", "1 2\na  "),
    ("(1 2;0w -0w)", "1  2  \n0w -0w"),
    ("(1.5 0n;2 3)", "1.5  \n2   3"),
    ("(1 2;\"a b\")", "1 2\n\"a b\""),
    ("(`a`b;\"cd\")", "a b\nc d"),
    ("1,2.5", "1\n2.5"),
    ("1 2,3.0", "1\n2\n3f"),
    ("1,\"a\"", "1\n\"a\""),
    ("(1;`a),2", "1\n`a\n2"),
    ("2#(1;`a;3)", "1\n`a"),
    ("(1;2 3) 1 0", "2 3\n1"),
    ("neg (1;2 3)", "-1\n-2 -3"),
    ("(1;2 3)+10", "11\n12 13"),
    ("(1;`a;3) 0 2", "1 3"),
    ("(1 2;3 4) 1 0", "3 4\n1 2"),
    ("(1;2.5)=(1;2.5)", "11b"),
    ("-1#(1;`a;3)", ",3"),
    ("1 2,(3;`a)", "1\n2\n3\n`a"),
];

#[test]
fn glist_matrix_cells_escape_characters() {
    // deliberate deviation: q prints a quote, backslash or tab raw inside a table cell; O
    // escapes them as it does in a string, so a control character cannot break the layout
    assert_eq!(shown("(1 2;\"a\\\"\")"), "1 2 \na \\\"");
    assert_eq!(shown("(1 2;\"a\\t\")"), "1 2 \na \\t");
    assert_eq!(shown("(1 2;\"a\\\\\")"), "1 2 \na \\\\");
}

#[test]
fn glist_display_one_per_line() {
    assert_eq!(shown("(1;`a;2.5)"), "1\n`a\n2.5");
    assert_eq!(shown("(\"abc\";1)"), "\"abc\"\n1");
}

#[test]
fn glist_display_matches_q() {
    for (src, expected) in DISPLAY {
        assert_eq!(&shown(src), expected, "{src}");
    }
}

#[test]
fn glist_singleton_paren_is_not_list() {
    assert_eq!(eval("(1)"), Atom::Integer(1));
    assert_eq!(eval("((1))"), Atom::Integer(1));
    assert_eq!(eval("(1 2)"), eval("1 2"));
    assert_eq!(eval("(`a)"), eval("`a"));
    assert_eq!(eval("(1+2)"), Atom::Integer(3));
    assert_eq!(eval("(x:4)"), Atom::Integer(4));
}

#[test]
fn glist_enlist() {
    // q: enlist 1 is the one-item long vector ,1 (an atom collapses) ...
    assert_eq!(eval("enlist 1"), eval("1#1"));
    assert_eq!(shown("enlist 1"), ",1");
    assert_eq!(shown("enlist `a"), ",`a");
    // ... and enlist of a vector is a one-item general list holding it.
    let v = eval("enlist 1 2");
    assert_eq!(items(&v), vec![eval("1 2")]);
    assert_eq!(eval("count enlist 1 2"), Atom::Integer(1));
    assert_eq!(eval("count enlist 1"), Atom::Integer(1));
    assert_eq!(shown("enlist 1 2"), "1 2");
    assert_eq!(eval("enlist 1 2").type_code(), 0);
    assert_eq!(eval("enlist 1").type_code(), 7);
    assert_eq!(items(&eval("enlist (1;`a)")), vec![eval("(1;`a)")]);
    assert_eq!(eval("enlist ()").type_code(), 0);
    // enlist takes everything to its right
    assert_eq!(eval("enlist 1+2"), eval("enlist 3"));
    assert_eq!(eval("enlist[1 2]"), eval("enlist 1 2"));
    // deliberate deviation: q's enlist is variadic, O's takes one argument
    assert_eq!(run("enlist[1;2]"), Err(QError::Rank));
    assert_eq!(run("enlist[]"), Err(QError::Rank));
}

#[test]
fn glist_evaluates_right_to_left() {
    let mut i = Interpreter::new();
    i.eval_line("x:5").unwrap();
    // q: x:5 first, then (x:1;x) gives 1 5, and x ends as 1
    assert_eq!(i.eval_line("(x:1;x)").unwrap(), Some(eval("1 5")));
    assert_eq!(i.eval_line("x").unwrap(), Some(eval("1")));
    i.eval_line("a:0").unwrap();
    assert_eq!(i.eval_line("(a:1;a:2;a)").unwrap(), Some(eval("1 2 0")));
    assert_eq!(i.eval_line("a").unwrap(), Some(eval("1")));
    // an error in the rightmost item stops evaluation before the others run
    assert_eq!(
        i.eval_line("(b:1;nope)"),
        Err(QError::Undefined("nope".into()))
    );
    assert_eq!(i.get("b"), None);
}

#[test]
fn glist_elided_items_are_nyi() {
    for src in ["(1;;2)", "(1;)", "(;1)", "(;)", "(;;)"] {
        assert_eq!(
            run(src),
            Err(QError::Nyi("elided list item".into())),
            "{src}"
        );
    }
}

#[test]
fn glist_parse_errors() {
    assert!(matches!(run("(1;2"), Err(QError::Parse(_))));
    assert!(matches!(run("(1;2;"), Err(QError::Parse(_))));
    assert!(matches!(run("(1 2"), Err(QError::Parse(_))));
    assert!(matches!(run("("), Err(QError::Parse(_))));
}

#[test]
fn glist_indexing() {
    let mut i = Interpreter::new();
    i.eval_line("l:(1;`a;2.5)").unwrap();
    let mut at = |src: &str| i.eval_line(src).unwrap().unwrap().to_string();
    assert_eq!(at("l 0"), "1");
    assert_eq!(at("l[1]"), "`a");
    assert_eq!(at("l 0 2"), "1\n2.5");
    assert_eq!(at("l[2 1 0]"), "2.5\n`a\n1");
    // out of range: the null of the first item's type (q 4.1)
    assert_eq!(at("l 5"), "0N");
    assert_eq!(at("l -1"), "0N");
    assert_eq!(at("l 0 5"), "1 0N"); // normalised back to a vector
    assert_eq!(at("l 1b"), "`a");
    assert_eq!(at("l[]"), "1\n`a\n2.5");
    // the null follows the first item: symbol, vector, nested list, empty list
    assert_eq!(at("(`a;1) 5"), "`");
    assert_eq!(at("(2.5;`a) 5"), "0n");
    assert_eq!(at("(1b;`a) 5"), "0b");
    assert_eq!(at("(\"a\";1) 5"), "\" \"");
    assert_eq!(at("(1 2;3 4) 5"), "`long$()");
    assert_eq!(at("(`a`b;1) 5"), "`symbol$()");
    assert_eq!(at("(\"ab\";1) 5"), "\"\"");
    assert_eq!(at("((1;`a);3) 5"), "0N\n`");
    assert_eq!(at("() 5"), "");
    assert_eq!(at("(1 2;3 4) 1"), "3 4");
    assert_eq!(at("(1 2;3 4) 1 0"), "3 4\n1 2");
    // an index list that is itself general indexes item by item
    assert_eq!(at("(1;`a) (1;0)"), "`a\n1");
    // errors as for vectors
    assert_eq!(i.eval_line("l 0.5"), Err(QError::Type));
    assert_eq!(i.eval_line("l `a"), Err(QError::Type));
    assert_eq!(
        i.eval_line("l[0;1]"),
        Err(QError::Nyi("depth indexing".into()))
    );
}

#[test]
fn glist_verbs_agree_with_q() {
    // every line here was run in q 4.1
    for (src, expected) in [
        ("count (1;2 3)", "2"),
        ("count ()", "0"),
        ("2#(1;`a;3)", "1\n`a"),
        ("5#(1;`a)", "1\n`a\n1\n`a\n1"),
        ("-1#(1;`a;3)", ",3"),
        ("0#(1;`a)", ""),
        ("(1;`a),2", "1\n`a\n2"),
        ("1 2,(3;`a)", "1\n2\n3\n`a"),
        ("(1;`a),(2;\"x\")", "1\n`a\n2\n\"x\""),
        ("1,2.5", "1\n2.5"),
        ("(1;2 3)+10", "11\n12 13"),
        ("neg (1;2 3)", "-1\n-2 -3"),
        ("(1;2.5)=(1;2.5)", "11b"),
        ("(1;2 3) 1 0", "2 3\n1"),
    ] {
        assert_eq!(shown(src), expected, "{src}");
    }
    assert_eq!(run("(1;`a)+1"), Err(QError::Type));
}

#[test]
fn glist_nesting_depth_is_capped() {
    // 64 levels are fine, the 65th is 'limit; the same through list literals
    for form in ["l:enlist l", "l:(l;1)", "l:(1;l)"] {
        let mut i = Interpreter::new();
        i.eval_line("l:1 2").unwrap();
        for _ in 0..64 {
            i.eval_line(form).unwrap();
        }
        assert!(matches!(i.eval_line(form), Err(QError::Limit(_))), "{form}");
        // the binding is untouched by the failed line, and the value still works
        assert_eq!(
            i.eval_line("count l").unwrap().unwrap(),
            Atom::Integer(if form == "l:enlist l" { 1 } else { 2 })
        );
        assert_eq!(i.eval_line("l=l").unwrap().unwrap().type_code(), 0);
        let _ = i.eval_line("l").unwrap().unwrap().to_string();
    }
    // sharing keeps a value shallow while doubling its logical size: capped at 1M logical items
    let mut i = Interpreter::new();
    i.eval_line("l:1 2").unwrap();
    let mut rounds = 0;
    while i.eval_line("l:(l;l)").is_ok() {
        rounds += 1;
        assert!(rounds < 30, "no limit");
    }
    assert_eq!(rounds, 18); // sizes 3 * 2^k - 1 stay under 1M until k = 19
    assert_eq!(i.eval_line("count l").unwrap().unwrap(), Atom::Integer(2));
}

#[test]
fn glist_names_a_list_and_reads_it_back() {
    let mut i = Interpreter::new();
    assert_eq!(
        i.eval_line("l:(1;`a)").unwrap().unwrap().to_string(),
        "1\n`a"
    );
    assert_eq!(i.eval_line("count l").unwrap(), Some(eval("2")));
    assert_eq!(i.eval_line("l,l").unwrap().unwrap().type_code(), 0);
}

// ---- a reference renderer, written from q's console rules, against random structures ----
//
// q formats a general list line by line. If any item is an atom, or the items differ in
// length, or they are all boolean (or all character) vectors, each item is its `-3!` form.
// Otherwise the items are rows of a matrix: each row is cut into cells (a vector into its
// elements without type marks, a nested general list by the same rule), every column is
// padded to its widest cell and the cells are joined by one space.

#[derive(Clone, Debug)]
enum Kind {
    Long,
    Sym,
    Float,
    Bool,
    Char,
}

#[derive(Clone, Debug)]
enum T {
    Atom {
        k: Kind,
        src: String,
        cell: String,
        repr: String,
    },
    Vec {
        k: Kind,
        src: String,
        cells: Vec<String>,
        repr: String,
    },
    List {
        src: String,
        items: Vec<T>,
    },
}

fn same_kind(a: &Kind, b: &Kind) -> bool {
    std::mem::discriminant(a) == std::mem::discriminant(b)
}

impl T {
    fn src(&self) -> &str {
        match self {
            T::Atom { src, .. } | T::Vec { src, .. } | T::List { src, .. } => src,
        }
    }
    /// The `-3!` string.
    fn repr(&self) -> String {
        match self {
            T::Atom { repr, .. } | T::Vec { repr, .. } => repr.clone(),
            T::List { items, .. } => match items.len() {
                0 => "()".into(),
                1 => format!(",{}", items[0].repr()),
                _ => format!(
                    "({})",
                    items.iter().map(T::repr).collect::<Vec<_>>().join(";")
                ),
            },
        }
    }
    fn len(&self) -> usize {
        match self {
            T::Atom { .. } => 1,
            T::Vec { cells, .. } => cells.len(),
            T::List { items, .. } => items.len(),
        }
    }
    /// q type code: 0 for a general list, positive for a vector.
    fn code(&self) -> u8 {
        match self {
            T::List { .. } => 0,
            T::Vec { k, .. } | T::Atom { k, .. } => match k {
                Kind::Bool => 1,
                Kind::Long => 7,
                Kind::Float => 9,
                Kind::Char => 10,
                Kind::Sym => 11,
            },
        }
    }
}

/// `(a;b;...)`: a list of atoms of one kind collapses to a vector.
fn make_list(items: Vec<T>) -> T {
    let src = format!(
        "({})",
        items.iter().map(T::src).collect::<Vec<_>>().join(";")
    );
    let atoms: Vec<(&Kind, &String, &String)> = items
        .iter()
        .filter_map(|t| match t {
            T::Atom { k, cell, repr, .. } => Some((k, cell, repr)),
            _ => None,
        })
        .collect();
    if items.len() >= 2
        && atoms.len() == items.len()
        && atoms.iter().all(|a| same_kind(a.0, atoms[0].0))
    {
        let k = atoms[0].0.clone();
        let reprs: Vec<&str> = atoms.iter().map(|a| a.2.as_str()).collect();
        let repr = match k {
            Kind::Bool => format!("{}b", reprs.iter().map(|r| &r[..1]).collect::<String>()),
            Kind::Char => format!("\"{}\"", reprs.iter().map(|r| &r[1..2]).collect::<String>()),
            Kind::Sym => reprs.concat(),
            _ => reprs.join(" "),
        };
        let cells = atoms.iter().map(|a| a.1.clone()).collect();
        return T::Vec {
            k,
            src,
            cells,
            repr,
        };
    }
    T::List { src, items }
}

fn cells(row: &T) -> Vec<String> {
    match row {
        T::Vec { cells, .. } => cells.clone(),
        T::List { items, .. } => lines(items),
        T::Atom { .. } => unreachable!("matrix rows are lists"),
    }
}

/// One string per item, as q's console builds them.
fn lines(items: &[T]) -> Vec<String> {
    if items.is_empty() {
        return vec![];
    }
    let flat = items.iter().any(|t| matches!(t, T::Atom { .. }))
        || items.iter().any(|t| t.len() != items[0].len())
        || (items.iter().all(|t| t.code() == items[0].code()) && matches!(items[0].code(), 1 | 10));
    if flat {
        return items.iter().map(T::repr).collect();
    }
    let rows: Vec<Vec<String>> = items.iter().map(cells).collect();
    let widths: Vec<usize> = (0..rows[0].len())
        .map(|c| rows.iter().map(|r| r[c].chars().count()).max().unwrap())
        .collect();
    rows.iter()
        .map(|r| {
            r.iter()
                .zip(&widths)
                .map(|(c, w)| format!("{c:<w$}"))
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect()
}

fn render(t: &T) -> String {
    match t {
        T::List { items, .. } => lines(items).join("\n"),
        other => other.repr(),
    }
}

fn atom(k: Kind, src: &str, cell: &str, repr: &str) -> T {
    T::Atom {
        k,
        src: src.into(),
        cell: cell.into(),
        repr: repr.into(),
    }
}

fn vector(k: Kind, src: &str, cells: &[&str], repr: &str) -> T {
    T::Vec {
        k,
        src: src.into(),
        cells: cells.iter().map(|c| c.to_string()).collect(),
        repr: repr.into(),
    }
}

fn leaf() -> impl Strategy<Value = T> {
    let leaves = vec![
        atom(Kind::Long, "1", "1", "1"),
        atom(Kind::Long, "22", "22", "22"),
        atom(Kind::Long, "0N", "", "0N"),
        atom(Kind::Sym, "`a", "a", "`a"),
        atom(Kind::Sym, "`bc", "bc", "`bc"),
        atom(Kind::Float, "1.5", "1.5", "1.5"),
        atom(Kind::Float, "2.25", "2.25", "2.25"),
        atom(Kind::Bool, "1b", "1", "1b"),
        atom(Kind::Bool, "0b", "0", "0b"),
        atom(Kind::Char, "\"x\"", "x", "\"x\""),
        vector(Kind::Long, "1 2", &["1", "2"], "1 2"),
        vector(Kind::Long, "10 0N 3", &["10", "", "3"], "10 0N 3"),
        vector(Kind::Sym, "`a`bc", &["a", "bc"], "`a`bc"),
        vector(Kind::Float, "1.5 2.25", &["1.5", "2.25"], "1.5 2.25"),
        vector(Kind::Bool, "101b", &["1", "0", "1"], "101b"),
        vector(Kind::Bool, "10b", &["1", "0"], "10b"),
        vector(Kind::Char, "\"ab\"", &["a", "b"], "\"ab\""),
        T::List {
            src: "()".into(),
            items: vec![],
        },
    ];
    prop::sample::select(leaves)
}

fn tree() -> impl Strategy<Value = T> {
    leaf().prop_recursive(3, 24, 4, |inner| {
        prop_oneof![
            4 => prop::collection::vec(inner.clone(), 2..=4).prop_map(make_list),
            1 => inner.prop_map(|t| {
                // `(enlist x)`: an atom becomes a one-item vector, anything else a one-item list
                let src = format!("(enlist {})", t.src());
                match t {
                    T::Atom { k, cell, repr, .. } => {
                        T::Vec { k, src, cells: vec![cell], repr: format!(",{repr}") }
                    }
                    other => T::List { src, items: vec![other] },
                }
            }),
        ]
    })
}

proptest! {
    #[test]
    fn glist_display_matches_reference(t in tree()) {
        // a bare top-level atom or vector is just its own form; the interesting
        // cases are lists, but those must hold at every depth the generator reaches
        let got = eval(t.src());
        // Empty `()` is no-print at the console, and displays as nothing
        prop_assert_eq!(got.to_string(), render(&t), "{}", t.src());
    }
}
