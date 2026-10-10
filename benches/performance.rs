use criterion::{
    black_box, criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion, Throughput,
};
use oxidedb::{Interpreter, Lexer, Parser};

/// `1+2+3+...` with `n` terms.
fn flat_chain(n: usize) -> String {
    (1..=n).map(|i| i.to_string()).collect::<Vec<_>>().join("+")
}

/// `((((1))))` nested `depth` deep.
fn nested_parens(depth: usize) -> String {
    format!("{}1{}", "(".repeat(depth), ")".repeat(depth))
}

/// Alternating long and float terms with all four operators.
fn mixed(n: usize) -> String {
    let ops = ['+', '*', '-', '%'];
    (1..=n)
        .map(|i| {
            if i % 2 == 0 {
                format!("{i}.5")
            } else {
                i.to_string()
            }
        })
        .enumerate()
        .fold(String::new(), |mut s, (i, t)| {
            if i > 0 {
                s.push(ops[i % ops.len()]);
            }
            s.push_str(&t);
            s
        })
}

/// Assignments followed by an expression that reads them back.
fn variable_heavy(n: usize) -> String {
    let mut s = String::new();
    for i in 0..n {
        let prev = if i == 0 {
            "1".into()
        } else {
            format!("v{}", i - 1)
        };
        s.push_str(&format!("v{i}:{prev}+1;"));
    }
    s
}

fn inputs() -> Vec<(&'static str, String)> {
    vec![
        ("simple", "2 + 3 * 4 - 1".into()),
        ("chain_500", flat_chain(500)),
        ("parens_100", nested_parens(100)),
        ("mixed_200", mixed(200)),
    ]
}

fn tokens(src: &str) -> Vec<oxidedb::language::lexer::Token> {
    Lexer::new(src).tokenize().unwrap()
}

fn benchmark_lexer(c: &mut Criterion) {
    let mut g = c.benchmark_group("lexer");
    for (name, src) in inputs() {
        g.throughput(Throughput::Bytes(src.len() as u64));
        g.bench_with_input(BenchmarkId::from_parameter(name), &src, |b, src| {
            b.iter(|| black_box(Lexer::new(black_box(src)).tokenize().unwrap()))
        });
    }
    g.finish();
}

fn benchmark_parser(c: &mut Criterion) {
    let mut g = c.benchmark_group("parser");
    for (name, src) in inputs() {
        let toks = tokens(&src);
        g.throughput(Throughput::Elements(toks.len() as u64));
        g.bench_with_input(BenchmarkId::from_parameter(name), &toks, |b, toks| {
            b.iter_batched(
                || toks.clone(),
                |t| black_box(Parser::new(t).parse().unwrap()),
                BatchSize::SmallInput,
            )
        });
    }
    g.finish();
}

fn benchmark_interpreter(c: &mut Criterion) {
    let mut g = c.benchmark_group("interpreter");
    for (name, src) in inputs() {
        let ast = Parser::new(tokens(&src)).parse().unwrap();
        g.bench_with_input(BenchmarkId::from_parameter(name), &ast, |b, ast| {
            b.iter_batched(
                || (Interpreter::new(), ast.clone()),
                |(mut interp, ast)| black_box(interp.evaluate(ast).unwrap()),
                BatchSize::SmallInput,
            )
        });
    }
    g.finish();
}

/// Full lex + parse + evaluate through `eval_line`, one line at a time.
fn benchmark_eval_line(c: &mut Criterion) {
    let mut g = c.benchmark_group("eval_line");
    for (name, src) in inputs() {
        g.throughput(Throughput::Bytes(src.len() as u64));
        g.bench_with_input(BenchmarkId::from_parameter(name), &src, |b, src| {
            b.iter_batched(
                Interpreter::new,
                |mut interp| black_box(interp.eval_line(black_box(src)).unwrap()),
                BatchSize::SmallInput,
            )
        });
    }
    // Variable-heavy: 100 dependent assignments in one session, then a read.
    let lines: Vec<String> = variable_heavy(100)
        .split(';')
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect();
    g.throughput(Throughput::Elements(lines.len() as u64));
    g.bench_function("variables_100", |b| {
        b.iter_batched(
            Interpreter::new,
            |mut interp| {
                for l in &lines {
                    black_box(interp.eval_line(l).unwrap());
                }
                black_box(interp.eval_line("v99").unwrap())
            },
            BatchSize::SmallInput,
        )
    });
    g.finish();
}

criterion_group!(
    benches,
    benchmark_lexer,
    benchmark_parser,
    benchmark_interpreter,
    benchmark_eval_line
);
criterion_main!(benches);
