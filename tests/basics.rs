use oxidedb::{Atom, Interpreter, Lexer, Parser};

#[test]
fn test_basic_arithmetic() {
    let mut lexer = Lexer::new("2 + 3");
    let tokens = lexer.tokenize().unwrap();

    let mut parser = Parser::new(tokens);
    let ast = parser.parse().unwrap();

    let mut interpreter = Interpreter::new();
    let result = interpreter.evaluate(ast).unwrap();

    assert_eq!(result, Atom::Integer(5));
}

#[test]
fn test_multiplication() {
    let mut lexer = Lexer::new("4 * 5");
    let tokens = lexer.tokenize().unwrap();

    let mut parser = Parser::new(tokens);
    let ast = parser.parse().unwrap();

    let mut interpreter = Interpreter::new();
    let result = interpreter.evaluate(ast).unwrap();

    assert_eq!(result, Atom::Integer(20));
}

#[test]
fn test_float_arithmetic() {
    let mut lexer = Lexer::new("2.5 + 1.5");
    let tokens = lexer.tokenize().unwrap();

    let mut parser = Parser::new(tokens);
    let ast = parser.parse().unwrap();

    let mut interpreter = Interpreter::new();
    let result = interpreter.evaluate(ast).unwrap();

    assert_eq!(result, Atom::Float(4.0));
}

#[test]
fn test_mixed_arithmetic() {
    let mut lexer = Lexer::new("2 + 3.5");
    let tokens = lexer.tokenize().unwrap();

    let mut parser = Parser::new(tokens);
    let ast = parser.parse().unwrap();

    let mut interpreter = Interpreter::new();
    let result = interpreter.evaluate(ast).unwrap();

    assert_eq!(result, Atom::Float(5.5));
}

#[test]
fn test_parentheses() {
    let mut lexer = Lexer::new("(2 + 3) * 4");
    let tokens = lexer.tokenize().unwrap();

    let mut parser = Parser::new(tokens);
    let ast = parser.parse().unwrap();

    let mut interpreter = Interpreter::new();
    let result = interpreter.evaluate(ast).unwrap();

    assert_eq!(result, Atom::Integer(20));
}

#[test]
fn test_right_to_left_evaluation() {
    // Q evaluates right-to-left: 1 + 2 * 2 + 1 = 1 + (2 * (2 + 1)) = 1 + (2 * 3) = 1 + 6 = 7
    let mut lexer = Lexer::new("1 + 2 * 2 + 1");
    let tokens = lexer.tokenize().unwrap();

    let mut parser = Parser::new(tokens);
    let ast = parser.parse().unwrap();

    let mut interpreter = Interpreter::new();
    let result = interpreter.evaluate(ast).unwrap();

    assert_eq!(result, Atom::Integer(7));
}

#[test]
fn test_right_to_left_subtraction() {
    // 4 - 2 + 1 = 4 - (2 + 1) = 4 - 3 = 1
    let mut lexer = Lexer::new("4 - 2 + 1");
    let tokens = lexer.tokenize().unwrap();

    let mut parser = Parser::new(tokens);
    let ast = parser.parse().unwrap();

    let mut interpreter = Interpreter::new();
    let result = interpreter.evaluate(ast).unwrap();

    assert_eq!(result, Atom::Integer(1));
}

#[test]
fn test_right_to_left_mixed_ops() {
    // 2 + 3 * 4 = 2 + (3 * 4) = 2 + 12 = 14
    let mut lexer = Lexer::new("2 + 3 * 4");
    let tokens = lexer.tokenize().unwrap();

    let mut parser = Parser::new(tokens);
    let ast = parser.parse().unwrap();

    let mut interpreter = Interpreter::new();
    let result = interpreter.evaluate(ast).unwrap();

    assert_eq!(result, Atom::Integer(14));
}

#[test]
fn test_variable_assignment() {
    let mut lexer = Lexer::new("x:5");
    let tokens = lexer.tokenize().unwrap();

    let mut parser = Parser::new(tokens);
    let ast = parser.parse().unwrap();

    let mut interpreter = Interpreter::new();
    let result = interpreter.evaluate(ast).unwrap();

    // Assignment should return the assigned value
    assert_eq!(result, Atom::Integer(5));
}

#[test]
fn test_variable_retrieval() {
    let mut interpreter = Interpreter::new();

    // First assign a variable
    let mut lexer = Lexer::new("x:10");
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse().unwrap();
    interpreter.evaluate(ast).unwrap();

    // Then retrieve it
    let mut lexer = Lexer::new("x");
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse().unwrap();
    let result = interpreter.evaluate(ast).unwrap();

    assert_eq!(result, Atom::Integer(10));
}

#[test]
fn test_variable_in_expression() {
    let mut interpreter = Interpreter::new();

    // Assign a variable
    let mut lexer = Lexer::new("y:7");
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse().unwrap();
    interpreter.evaluate(ast).unwrap();

    // Use it in an expression
    let mut lexer = Lexer::new("y * 3 + 1");
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse().unwrap();
    let result = interpreter.evaluate(ast).unwrap();

    // y * 3 + 1 = y * (3 + 1) = 7 * 4 = 28 (right-to-left evaluation)
    assert_eq!(result, Atom::Integer(28));
}

#[test]
fn test_variable_reassignment() {
    let mut interpreter = Interpreter::new();

    // First assignment
    let mut lexer = Lexer::new("z:3");
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse().unwrap();
    interpreter.evaluate(ast).unwrap();

    // Reassignment
    let mut lexer = Lexer::new("z:z+2");
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse().unwrap();
    let result = interpreter.evaluate(ast).unwrap();

    assert_eq!(result, Atom::Integer(5));

    // Verify the variable was updated
    let mut lexer = Lexer::new("z");
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse().unwrap();
    let result = interpreter.evaluate(ast).unwrap();

    assert_eq!(result, Atom::Integer(5));
}

#[test]
fn test_undefined_variable_error() {
    let mut lexer = Lexer::new("undefined_var");
    let tokens = lexer.tokenize().unwrap();

    let mut parser = Parser::new(tokens);
    let ast = parser.parse().unwrap();

    let mut interpreter = Interpreter::new();
    let result = interpreter.evaluate(ast);

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Undefined variable"));
}

#[test]
fn test_negative_numbers() {
    let mut lexer = Lexer::new("-5 + 3");
    let tokens = lexer.tokenize().unwrap();

    let mut parser = Parser::new(tokens);
    let ast = parser.parse().unwrap();

    let mut interpreter = Interpreter::new();
    let result = interpreter.evaluate(ast).unwrap();

    assert_eq!(result, Atom::Integer(-2));
}
