use criterion::{black_box, criterion_group, criterion_main, Criterion};
use oxidedb::{Lexer, Parser, Interpreter};

fn benchmark_lexer(c: &mut Criterion) {
    c.bench_function("lexer_simple_expression", |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box("2 + 3 * 4 - 1"));
            lexer.tokenize().unwrap()
        })
    });
}

fn benchmark_parser(c: &mut Criterion) {
    let mut lexer = Lexer::new("2 + 3 * 4 - 1");
    let tokens = lexer.tokenize().unwrap();
    
    c.bench_function("parser_simple_expression", |b| {
        b.iter(|| {
            let mut parser = Parser::new(black_box(tokens.clone()));
            parser.parse().unwrap()
        })
    });
}

fn benchmark_interpreter(c: &mut Criterion) {
    let mut lexer = Lexer::new("2 + 3 * 4 - 1");
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse().unwrap();
    
    c.bench_function("interpreter_simple_expression", |b| {
        b.iter(|| {
            let mut interpreter = Interpreter::new();
            interpreter.evaluate(black_box(ast.clone())).unwrap()
        })
    });
}

criterion_group!(benches, benchmark_lexer, benchmark_parser, benchmark_interpreter);
criterion_main!(benches);