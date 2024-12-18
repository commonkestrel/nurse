use std::{fs, process::ExitCode};

use logos::{Lexer, Logos};
use nurse::{
    error,
    reporter::{Reporter, SerialReporter},
    span::{Span, Spanned},
    spanned_error,
};

const PHI: f64 = 1.618033988749894848204586834365638118_f64;

fn main() -> ExitCode {
    let file = fs::read_to_string("examples/serial.txt").expect("unable to open `serial.txt`");

    let mut reporter = SerialReporter::default();
    let lookup = reporter.register_file("serial.txt", file.clone());
    let eof = reporter.eof_span(lookup);

    let mut tokens = Vec::new();

    let lex: Lexer<Token> = Lexer::new(file.as_str());
    for (token, range) in lex.spanned() {
        let span = Span::new(lookup, range);

        match token {
            Ok(tok) => tokens.push(Spanned::new(tok, span)),
            Err(err) => reporter.report(if err.is_empty() {
                spanned_error!(span, "unknown token")
            } else {
                spanned_error!(span, "{err}")
            }),
        }
    }

    if reporter.has_errors() {
        reporter.report(error!("unable to compile due to previous syntax errors"));
        reporter.emit_all(&mut std::io::stdout());

        return ExitCode::FAILURE;
    }

    let expr = parse_expression(&tokens, &mut 0, &mut reporter, eof);
    if reporter.has_errors() {
        reporter.report(error!("unable to compile due to previous syntax errors"));
        reporter.emit_all(&mut std::io::stdout());

        return ExitCode::FAILURE;
    }

    println!("Result: {}", expr.into_inner().eval());

    ExitCode::SUCCESS
}

#[derive(Logos, Debug, Clone, Copy)]
#[logos(error = String)]
#[logos(skip r"[ \t\f\n\r]")]
#[logos(skip r"//[^!][^\n]*\n?")]
#[logos(skip r"/\*(?:[^*]|\*[^/])*\*/")]
enum Token {
    #[regex(r"-?[0-9]+(\.[0-9]*)?", parse_float)]
    Number(f64),
    #[token("pi", |_| Constant::Pi)]
    #[token("e", |_| Constant::E)]
    #[token("tau", |_| Constant::Tau)]
    #[token("phi", |_| Constant::Phi)]
    Constant(Constant),
    #[token("sin", |_| Function::Sin)]
    #[token("cos", |_| Function::Cos)]
    #[token("tan", |_| Function::Tan)]
    #[token("sqrt", |_| Function::Sqrt)]
    #[token("cbrt", |_| Function::Cbrt)]
    #[token("ln", |_| Function::Ln)]
    Function(Function),
    #[token("(")]
    OpenParen,
    #[token(")")]
    CloseParen,
    #[token("^")]
    Carat,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
}

impl Token {
    fn name(&self) -> &'static str {
        match self {
            Token::Number(_) => "number",
            Token::Constant(constant) => constant.name(),
            Token::Function(func) => func.name(),
            Token::OpenParen => "(",
            Token::CloseParen => ")",
            Token::Carat => "^",
            Token::Star => "*",
            Token::Slash => "/",
            Token::Plus => "+",
            Token::Minus => "-",
        }
    }
}

fn parse_float(lex: &mut Lexer<Token>) -> Result<f64, String> {
    let slice = lex.slice();
    match slice.parse() {
        Ok(num) => Ok(num),
        Err(err) => Err(format!("unable to parse number; {err}")),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Constant {
    Pi,
    E,
    Tau,
    Phi,
}

impl Constant {
    fn name(&self) -> &'static str {
        match self {
            Constant::Pi => "pi",
            Constant::E => "e",
            Constant::Tau => "tau",
            Constant::Phi => "phi",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Function {
    Sin,
    Cos,
    Tan,
    Sqrt,
    Cbrt,
    Ln,
}

impl Function {
    fn name(&self) -> &'static str {
        match self {
            Function::Sin => "sin",
            Function::Cos => "cos",
            Function::Tan => "tan",
            Function::Sqrt => "sqrt",
            Function::Cbrt => "cbrt",
            Function::Ln => "ln",
        }
    }
}

#[derive(Debug)]
enum Expr {
    Binary(Box<BinaryOp>),
    Unary {
        value: Box<Spanned<Expr>>,
        op: UnaryOp,
    },
    Number(f64),
    Err,
}

impl Expr {
    fn eval(self) -> f64 {
        match self {
            Expr::Binary(binop) => binop.eval(),
            Expr::Unary { value, op } => match op {
                UnaryOp::Negative => -value.into_inner().eval(),
                UnaryOp::Sqrt => value.into_inner().eval().sqrt(),
                UnaryOp::Cbrt => value.into_inner().eval().cbrt(),
                UnaryOp::Ln => value.into_inner().eval().ln(),
                UnaryOp::Sin => value.into_inner().eval().sin(),
                UnaryOp::Cos => value.into_inner().eval().cos(),
                UnaryOp::Tan => value.into_inner().eval().tan(),
            }
            Expr::Number(val) => val,
            // Errors should be dealt with when checking the reporter for errors
            Expr::Err => unreachable!(),
        }
    }
}

#[derive(Debug)]
struct BinaryOp {
    left: Spanned<Expr>,
    right: Spanned<Expr>,
    op: BinOp,
}

impl BinaryOp {
    #[inline]
    fn boxed(left: Spanned<Expr>, right: Spanned<Expr>, op: BinOp) -> Box<BinaryOp> {
        Box::new(BinaryOp{ left, right, op })
    }

    fn eval(self) -> f64 {
        let left = self.left.into_inner().eval();
        let right = self.right.into_inner().eval();

        match self.op {
            BinOp::Plus => left + right,
            BinOp::Minus => left - right,
            BinOp::Mult => left * right,
            BinOp::Div => left / right,
            BinOp::Power => left.powf(right),
        }
    }
}

#[derive(Debug)]
enum BinOp {
    Plus,
    Minus,
    Mult,
    Div,
    Power,
}

#[derive(Debug)]
enum UnaryOp {
    Negative,
    Sin,
    Cos,
    Tan,
    Sqrt,
    Cbrt,
    Ln,
}

fn parse_expression(stream: &Vec<Spanned<Token>>, index: &mut usize, reporter: &mut impl Reporter, eof: Span) -> Spanned<Expr> {
    let mut a = parse_terminal(stream, index, reporter, eof);

    while let Some(tok) = stream.get(*index).as_deref() {
        match tok.inner() {
            Token::Plus => {
                *index += 1;
                let b = parse_terminal(stream, index, reporter, eof);
                let span = a.span().to(b.span());
                let binop = BinaryOp::boxed(a, b, BinOp::Plus);
                a = Spanned::new(Expr::Binary(binop), span);
            }
            Token::Minus => {
                *index += 1;
                let b = parse_terminal(stream, index, reporter, eof);
                let span = a.span().to(b.span());
                let binop = BinaryOp::boxed(a, b, BinOp::Minus);
                a = Spanned::new(Expr::Binary(binop), span);
            }
            _ => return a,
        }
    }

    return a;
}

fn parse_terminal(stream: &Vec<Spanned<Token>>, index: &mut usize, reporter: &mut impl Reporter, eof: Span) -> Spanned<Expr> {
    let mut a = parse_factor(stream, index, reporter, eof);

    while let Some(tok) = stream.get(*index).as_deref() {
        match tok.inner() {
            Token::Star => {
                *index += 1;
                let b = parse_factor(stream, index, reporter, eof);
                let span = a.span().to(b.span());
                let binop = BinaryOp::boxed(a, b, BinOp::Mult);
                a = Spanned::new(Expr::Binary(binop), span);
            }
            Token::Slash => {
                *index += 1;
                let b = parse_factor(stream, index, reporter, eof);
                let span = a.span().to(b.span());
                let binop = BinaryOp::boxed(a, b, BinOp::Div);
                a = Spanned::new(Expr::Binary(binop), span);
            }
            Token::Carat => {
                *index += 1;
                let b = parse_factor(stream, index, reporter, eof);
                let span = a.span().to(b.span());
                let binop = BinaryOp::boxed(a, b, BinOp::Power);
                a = Spanned::new(Expr::Binary(binop), span);
            }
            Token::OpenParen => {
                let start_span = tok.span();

                *index += 1;
                let b = parse_expression(stream, index, reporter, eof);

                let next = stream.get(*index);
                if let Some(tok) = next {
                    *index += 1;
                    if !matches!(tok.inner(), Token::CloseParen) {
                        reporter.report(spanned_error!(*tok.span(), "unexpected token `{}`; expected closing parenthesis", tok.name()));
                        return Spanned::new(Expr::Err, *tok.span());
                    }
                } else {
                    reporter.report(spanned_error!(*start_span, "unmatched opening parenthesis"));
                    return Spanned::new(Expr::Err, *start_span);
                }

                let span = a.span().to(b.span());
                let binop = BinaryOp::boxed(a, b, BinOp::Mult);
                a = Spanned::new(Expr::Binary(binop), span);
            }
            _ => return a,
        }
    }

    return a;
}

fn parse_factor(stream: &Vec<Spanned<Token>>, index: &mut usize, reporter: &mut impl Reporter, eof: Span) -> Spanned<Expr> {
    match stream.get(*index) {
        Some(tok) => {
            *index += 1;

            match tok.inner() {
                Token::Number(num) => Spanned::new(Expr::Number(*num), *tok.span()),
                Token::Constant(constant) => Spanned::new(Expr::Number(match constant {
                    Constant::E => std::f64::consts::E,
                    Constant::Pi => std::f64::consts::PI,
                    Constant::Tau => std::f64::consts::TAU,
                    Constant::Phi => PHI,
                }), *tok.span()),
                Token::Minus => {
                    let value = parse_factor(stream, index, reporter, eof);
                    let span = tok.span().to(value.span());
                    let expr = Expr::Unary {
                        op: UnaryOp::Negative,
                        value: Box::new(value),
                    };

                    Spanned::new(expr, span)
                }
                Token::Function(func) => {
                    let op = match func {
                        Function::Sqrt => UnaryOp::Sqrt,
                        Function::Cbrt => UnaryOp::Cbrt,
                        Function::Ln => UnaryOp::Ln,
                        Function::Sin => UnaryOp::Sin,
                        Function::Cos => UnaryOp::Cos,
                        Function::Tan => UnaryOp::Tan,
                    };

                    let value = parse_factor(stream, index, reporter, eof);
                    let span = tok.span().to(value.span());
                    let expr = Expr::Unary {
                        op, value: Box::new(value),
                    };

                    Spanned::new(expr, span)
                }
                Token::OpenParen => {
                    let expr = parse_expression(stream, index, reporter, eof);

                    let maybe_next = stream.get(*index);
                    let span = if let Some(next) = maybe_next {
                        *index += 1;
                        if !matches!(next.inner(), Token::CloseParen) {
                            reporter.report(spanned_error!(*next.span(), "unexpected token `{}`; expected closing parenthesis", next.name()));
                            return Spanned::new(Expr::Err, *next.span());
                        }

                        tok.span().to(next.span())
                    } else {
                        reporter.report(spanned_error!(*tok.span(), "unmatched opening parenthesis"));
                        return Spanned::new(Expr::Err, *tok.span());
                    };

                    Spanned::new(expr.into_inner(), span)
                }
                _ => {
                    reporter.report(spanned_error!(*tok.span(), "unexpected token {}", tok.name()));
                    Spanned::new(Expr::Err, *tok.span())
                }
            }
        }
        None => {
            reporter.report(spanned_error!(eof, "expected expression, found `eof`"));
            return Spanned::new(Expr::Err, eof);
        }
    }
}
