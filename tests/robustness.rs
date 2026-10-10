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
            "0W", "1e", "1F",
        ][..]).prop_map(String::from),
        2 => prop::sample::select(&["a", "é", "_", "x1", "日本"][..]).prop_map(String::from),
    ]
}

fn chain(atom: impl Strategy<Value = String>) -> impl Strategy<Value = String> {
    let op = prop::sample::select(&["+", "-", "*", "%"][..]);
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
            "101b",
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
    "(1)",
    "-(1)",
    "-(1.5)",
    "-(9223372036854775807)",
    // Monadic minus on the reserved null (the long null `0N`): must never panic.
    "-(-9223372036854775808)",
];
const OPS: &[&str] = &["+", "-", "*", "%"];

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
