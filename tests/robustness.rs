use oxidedb::{Interpreter, Lexer, Parser};
use proptest::prelude::*;

// Mostly well-formed expressions (so cases reach the parser and interpreter), with a little
// noise from tokens that are unlexable or ungrammatical. Always cut to 16 tokens, which keeps
// nesting far below the deep-nesting limit.
fn operand() -> impl Strategy<Value = String> {
    prop_oneof![
        6 => prop::sample::select(&[
            "0", "00", "0.", "1", "1.5", "42", "9223372036854775807", "9223372036854775808",
            "-9223372036854775808", "1b", "0b", "-1", "0N", "0n", "0w", "-0w", "1e3", "1e-3", "1f",
            "0W", "1e", "1F", "1 2 3", "1 2.5 3", "1 -2", "0N 0n", "1 0N", "101b", "`a`b",
            "til 3", "count til 4", "neg 1 2", "1 2 3[1]", "(1 2 3)[0 5]", "til[2]", "count[1;2]",
            "1 2 3 1", "\"abc\" 0 7", "x[0]:1", "x[1 9]:2", "(x)[0]:x[1]:5", "x[0]+:1", "x[0][0]:1", "neg 0N", "til -1", "til 0N", "`a",
            "\"abc\"", "1 1b", "1 \"a\"", "1f 2", "1 2 3f", "1 2 + 3 4",
        ][..]).prop_map(String::from),
        2 => prop::sample::select(&["a", "é", "_", "x1", "日本"][..]).prop_map(String::from),
    ]
}

fn chain(atom: impl Strategy<Value = String>) -> impl Strategy<Value = String> {
    let op = prop::sample::select(
        &[
            "+", "-", "*", "%", "=", "<", ">", "<>", "<=", ">=", "#", ",", "!",
        ][..],
    );
    (
        atom,
        proptest::collection::vec((op, any::<bool>(), operand()), 0..=3),
    )
        .prop_map(|(first, rest)| {
            let mut out = first;
            for (op, neg, x) in rest {
                out = format!("{out} {op} {}{x}", if neg { "- " } else { "" });
            }
            out
        })
}

fn noise() -> impl Strategy<Value = &'static str> {
    prop::sample::select(
        &[
            ":", ";", "[", "]", "(", ")", "\"", "\"a\"", "/", "\\", "'a'", "'", "`", "`sym", "+",
            "-", "%", "=", "<", ">", "<>", "<=", ">=", "#", ",", "!", "{", "}", "$", "@", "':",
            "101b", "/:", "\\:", "::", "\"ab\"", "\"\"", "\"\\n\"", "\"\\\"\"", "`a", "`a`b", "`",
        ][..],
    )
}

fn expression() -> impl Strategy<Value = Vec<String>> {
    let paren = chain(operand()).prop_map(|s| format!("( {s} )"));
    let e = prop_oneof![3 => chain(operand()), 2 => chain(paren), 1 => chain(operand()).prop_map(|s| format!("x : {s}"))];
    (e, proptest::collection::vec((0usize..24, noise()), 0..=1)).prop_map(|(e, ns)| {
        let mut toks: Vec<String> = e.split_whitespace().map(String::from).collect();
        for (at, n) in ns {
            toks.insert(at % (toks.len() + 1), n.to_string());
        }
        toks.truncate(16);
        toks
    })
}

const NUMS: &[&str] = &[
    "0",
    "1",
    "2",
    "1.5",
    "0.",
    "9223372036854775807",
    "-9223372036854775807",
    "0b",
    "1 2 3",
    "1 2.5 0N",
    "(1 2)",
    "(1)",
    "-(1)",
    "-(1.5)",
    "-(9223372036854775807)",
    // Monadic minus on the reserved null (the long null `0N`): must never panic.
    "-(-9223372036854775808)",
];
const OPS: &[&str] = &[
    "+", "-", "*", "%", "=", "<", ">", "<>", "<=", ">=", "#", ",", "!",
];

/// Lexer -> Parser -> Interpreter -> Display, ignoring every `Err`; only a panic fails.
fn pipeline(src: &str) {
    stage(src);
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum Stage {
    LexErr,
    ParseErr,
    EvalErr,
    Ok { tokens: usize },
}

fn stage(src: &str) -> Stage {
    let Ok(tokens) = Lexer::new(src).tokenize() else {
        return Stage::LexErr;
    };
    let n = tokens.len();
    let Ok(ast) = Parser::new(tokens).parse() else {
        return Stage::ParseErr;
    };
    match Interpreter::new().evaluate(ast) {
        Ok(atom) => {
            let _ = atom.to_string();
            Stage::Ok { tokens: n }
        }
        Err(_) => Stage::EvalErr,
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1024))]

    #[test]
    fn arbitrary_unicode_never_panics(s in "\\PC{0,64}") {
        pipeline(&s);
    }

    // At most 16 tokens, so the deep-nesting limit is out of reach.
    #[test]
    fn token_soup_never_panics(v in expression()) {
        pipeline(&v.join(" "));
        pipeline(&v.concat());
    }

    // Alternating operand/operator (<= 16 tokens): well-formed enough to reach overflow
    // and float edge cases, which random soup almost never parses to.
    #[test]
    fn arithmetic_chain_never_panics(
        v in proptest::collection::vec((0..NUMS.len(), 0..OPS.len()), 0..=8)
    ) {
        let src: String = v.iter().map(|&(n, o)| format!("{}{}", NUMS[n], OPS[o])).collect();
        pipeline(&format!("{src}1"));
    }
}

const APPLY: &[&str] = &[
    "til", "count", "neg", "x", "v", "1", "0", "-1", "1 2", "0N", "`a", "\"ab\"", "101b", "3.5",
    "(", ")", "[", "]", ";", "+", "-", "=", ",", ":", "x:", "v:1 2 3", "v[0]:", "v[0 1]:", "v[9]:",
    "v[-1]:", "v[0N]:", "v[101b]:", "x[0]:", "w[0]:", "v[0]+:",
];

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1024))]

    // Juxtaposition, brackets and builtin names in any order: never a panic.
    #[test]
    fn application_soup_never_panics(v in proptest::collection::vec(0..APPLY.len(), 0..=14)) {
        let toks: Vec<&str> = v.iter().map(|&i| APPLY[i]).collect();
        pipeline(&toks.join(" "));
        pipeline(&toks.concat());
    }

    // The same with `v` and `x` bound, so indexing paths run too.
    #[test]
    fn application_with_bindings_never_panics(v in proptest::collection::vec(0..APPLY.len(), 0..=14)) {
        let mut interp = Interpreter::new();
        let _ = interp.eval_line("v:10 20 30");
        let _ = interp.eval_line("x:5");
        let toks: Vec<&str> = v.iter().map(|&i| APPLY[i]).collect();
        if let Ok(Some(value)) = interp.eval_line(&toks.join(" ")) {
            let _ = value.to_string();
        }
    }
}

const AMEND: &[&str] = &[
    "v[0]:5",
    "v[2]:5",
    "v[3]:5",
    "v[0N]:5",
    "v[0 2]:7 8",
    "v[0 1 2]:7 8",
    "v[0 2]:9",
    "v[1.]:5",
    "v[0]:1.5",
    "v[0]:`a",
    "v[0]:0n",
    "v[0]:0N",
    "v[01b]:0 1",
    "v[0#0]:5",
    "v[0]:7 8",
    "l[0]:1b",
    "l[0 1]:3",
    "l[1]:2 3",
    "x[0]:1",
    "u[0]:1",
    "v",
    "l",
    "w:v",
    "v[0]:v[1]:3",
    "v[1]:v:5 6 7",
    "v[0]+:1",
    "v[0][0]:2",
    "count[0]:1",
    "s[0]:\"x\"",
    "s[0 1]:\"x\"",
    "s[0]:`x",
    "s",
];

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1024))]

    // Index assignments in any order, on a session holding every column type: never a panic,
    // and every binding still prints.
    #[test]
    fn index_assignment_sequences_never_panic(v in proptest::collection::vec(0..AMEND.len(), 0..=10)) {
        let mut interp = Interpreter::new();
        for setup in ["v:10 20 30", "x:5", "s:\"abc\""] {
            interp.eval_line(setup).unwrap();
        }
        interp.set("l", oxidedb::Value::List(std::rc::Rc::new(vec![
            oxidedb::Value::Atom(oxidedb::types::atom::Atom::Integer(1)),
            oxidedb::Value::Atom(oxidedb::types::atom::Atom::Boolean(true)),
        ])));
        for &i in &v {
            if let Ok(Some(value)) = interp.eval_line(AMEND[i]) {
                let _ = value.to_string();
            }
        }
        for name in ["v", "x", "s", "l", "w"] {
            if let Some(value) = interp.get(name) {
                let _ = value.to_string();
            }
        }
    }
}

/// Guards the generator itself: if few cases reach the interpreter the properties are hollow.
#[test]
fn token_soup_reaches_interpreter() {
    use proptest::strategy::ValueTree;
    use proptest::test_runner::TestRunner;
    let mut r = TestRunner::deterministic();
    let strat = expression();
    let (mut l, mut p, mut e, mut ok) = (0, 0, 0, 0);
    for _ in 0..10_000 {
        let v = strat.new_tree(&mut r).unwrap().current();
        match stage(&v.join(" ")) {
            Stage::LexErr => l += 1,
            Stage::ParseErr => p += 1,
            Stage::EvalErr => e += 1,
            Stage::Ok { tokens } if tokens >= 3 => ok += 1,
            Stage::Ok { .. } => {}
        }
    }
    println!("lex={l} parse={p} eval={e} ok_multi={ok}");
    assert!(
        ok >= 300,
        "only {ok}/10000 cases evaluated to a multi-token value"
    );
}
