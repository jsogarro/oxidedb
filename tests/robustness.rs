use oxidedb::{Interpreter, Lexer, Parser};
use proptest::prelude::*;

const TOKENS: &[&str] = &[
    "0",
    "00",
    "0.",
    "1",
    "1.5",
    "42",
    "9223372036854775807",
    "9223372036854775808",
    "-9223372036854775808",
    "1b",
    "0b",
    "-1",
    "a",
    "é",
    "_",
    "x1",
    "日本",
    ":",
    "%",
    "+",
    "-",
    "*",
    "(",
    ")",
    "[",
    "]",
    ";",
    "\"",
    "\"a\"",
    "'a'",
    "'",
    "`",
    "`sym",
    "/",
    "\\",
    " ",
];

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
];
const OPS: &[&str] = &["+", "-", "*", "%"];

/// Lexer -> Parser -> Interpreter -> Display, ignoring every `Err`; only a panic fails.
fn pipeline(src: &str) {
    let Ok(tokens) = Lexer::new(src).tokenize() else {
        return;
    };
    let Ok(ast) = Parser::new(tokens).parse() else {
        return;
    };
    if let Ok(atom) = Interpreter::new().evaluate(ast) {
        let _ = atom.to_string();
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
    fn token_soup_never_panics(v in proptest::collection::vec(0..TOKENS.len(), 0..=16)) {
        let src: String = v.iter().map(|&i| TOKENS[i]).collect::<Vec<_>>().join("");
        pipeline(&src);
        let spaced: String = v.iter().map(|&i| TOKENS[i]).collect::<Vec<_>>().join(" ");
        pipeline(&spaced);
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
