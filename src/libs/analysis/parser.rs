use std::collections::VecDeque;

use anyhow::{Result, bail, ensure};
use thiserror::Error;

use crate::syntax::ast::Expression;

#[derive(Copy, Clone, Eq, PartialEq)]
enum Assoc {
    Left,
    Right,
}

#[derive(Debug, Error)]
enum ParseError {
    #[error("Empty")]
    Empty,
    #[error("Unexpected token: {0}")]
    Unexpected(String),
    #[error("Invalid syntax: {0}")]
    InvalidSyntax(String),
    #[error("Unclosed input")]
    Unclosed,
}

pub fn parse(mut tokens: VecDeque<String>) -> Result<Expression> {
    let expr = parse_expr(&mut tokens, 0)?;
    ensure!(
        tokens.is_empty(),
        ParseError::Unexpected(tokens.pop_front().unwrap())
    );
    Ok(expr)
}

fn peek(tokens: &VecDeque<String>) -> Option<&str> {
    tokens.front().map(|s| s.as_str())
}

fn next(tokens: &mut VecDeque<String>) -> Option<String> {
    tokens.pop_front()
}

fn expect(tokens: &mut VecDeque<String>, expected: &str) -> Result<()> {
    let t = next(tokens).ok_or(ParseError::Empty)?;
    if t != expected {
        bail!(ParseError::Unexpected(t))
    }
    Ok(())
}

fn is_identifier(tok: &str) -> bool {
    let mut chars = tok.chars();
    match chars.next() {
        Some(c) if c.is_ascii_lowercase() => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn starts_primary(tokens: &VecDeque<String>) -> bool {
    match peek(tokens) {
        Some("(") | Some("[]") | Some("true") | Some("false") => true,
        Some(s) if s.parse::<isize>().is_ok() => true,
        Some(s) if is_identifier(s) => true,
        _ => false,
    }
}

fn precedence(op: &str) -> Option<(i32, Assoc)> {
    match op {
        "*" => Some((70, Assoc::Left)),
        "+" | "-" => Some((60, Assoc::Left)),
        "<" => Some((50, Assoc::Left)),
        "::" => Some((40, Assoc::Right)),
        _ => None,
    }
}

fn build_binop(op: &str, lhs: Expression, rhs: Expression) -> Result<Expression> {
    Ok(match op {
        "+" => Expression::Plus {
            expression1: Box::new(lhs),
            expression2: Box::new(rhs),
        },
        "-" => Expression::Minus {
            expression1: Box::new(lhs),
            expression2: Box::new(rhs),
        },
        "*" => Expression::Times {
            expression1: Box::new(lhs),
            expression2: Box::new(rhs),
        },
        "<" => Expression::LessThan {
            expression1: Box::new(lhs),
            expression2: Box::new(rhs),
        },
        "::" => Expression::Cons {
            car: Box::new(lhs),
            cdr: Box::new(rhs),
        },
        _ => bail!(ParseError::InvalidSyntax(op.to_owned())),
    })
}

fn parse_expr(tokens: &mut VecDeque<String>, min_bp: i32) -> Result<Expression> {
    let mut lhs = match peek(tokens) {
        Some("if") => parse_if(tokens)?,
        Some("let") => parse_let(tokens)?,
        Some("fun") => parse_fun(tokens)?,
        Some("match") => parse_match(tokens)?,
        _ => parse_application(tokens)?,
    };
    loop {
        let op_s = match peek(tokens) {
            Some(op) if precedence(op).is_some() => op.to_string(),
            _ => break,
        };
        let (bp, assoc) = precedence(&op_s).unwrap();
        if bp < min_bp {
            break;
        }
        let next_min = if assoc == Assoc::Left { bp + 1 } else { bp };
        next(tokens);
        let rhs = parse_expr(tokens, next_min)?;
        lhs = build_binop(&op_s, lhs, rhs)?;
    }
    Ok(lhs)
}

fn parse_if(tokens: &mut VecDeque<String>) -> Result<Expression> {
    expect(tokens, "if")?;
    let pred = parse_expr(tokens, 0)?;
    expect(tokens, "then")?;
    let cons = parse_expr(tokens, 0)?;
    expect(tokens, "else")?;
    let alt = parse_expr(tokens, 0)?;
    Ok(Expression::If {
        predicate: Box::new(pred),
        consequent: Box::new(cons),
        alternative: Box::new(alt),
    })
}

fn parse_let(tokens: &mut VecDeque<String>) -> Result<Expression> {
    expect(tokens, "let")?;

    if matches!(peek(tokens), Some("rec")) {
        next(tokens);

        let name = next(tokens).ok_or(ParseError::Empty)?;
        ensure!(
            is_identifier(&name),
            ParseError::InvalidSyntax(name.clone()).to_string()
        );
        expect(tokens, "=")?;

        expect(tokens, "fun")?;
        let param = next(tokens).ok_or(ParseError::Empty)?;
        ensure!(
            is_identifier(&param),
            ParseError::InvalidSyntax(param.clone()).to_string()
        );
        expect(tokens, "->")?;
        let body_fun = parse_expr(tokens, 0)?;
        let fun_expr = Expression::Fun {
            parameter: param,
            body: Box::new(body_fun),
        };

        expect(tokens, "in")?;
        let body = parse_expr(tokens, 0)?;
        return Ok(Expression::LetRec {
            variable: name,
            bound_function: Box::new(fun_expr),
            body: Box::new(body),
        });
    }

    let name = next(tokens).ok_or(ParseError::Empty)?;
    ensure!(
        is_identifier(&name),
        ParseError::InvalidSyntax(name.clone()).to_string()
    );
    expect(tokens, "=")?;
    let bound = parse_expr(tokens, 0)?;
    expect(tokens, "in")?;
    let body = parse_expr(tokens, 0)?;
    Ok(Expression::Let {
        variable: name,
        bound: Box::new(bound),
        body: Box::new(body),
    })
}

fn parse_fun(tokens: &mut VecDeque<String>) -> Result<Expression> {
    expect(tokens, "fun")?;
    let param = next(tokens).ok_or(ParseError::Empty)?;
    ensure!(
        is_identifier(&param),
        ParseError::InvalidSyntax(param.clone()).to_string()
    );
    expect(tokens, "->")?;
    let body = parse_expr(tokens, 0)?;
    Ok(Expression::Fun {
        parameter: param,
        body: Box::new(body),
    })
}

fn parse_match(tokens: &mut VecDeque<String>) -> Result<Expression> {
    expect(tokens, "match")?;
    let scrutinee = parse_expr(tokens, 0)?;
    expect(tokens, "with")?;

    expect(tokens, "[]")?;
    expect(tokens, "->")?;
    let nil_case = parse_expr(tokens, 0)?;
    expect(tokens, "|")?;
    let hd = next(tokens).ok_or(ParseError::Empty)?;
    ensure!(
        is_identifier(&hd),
        ParseError::InvalidSyntax(hd.clone()).to_string()
    );
    expect(tokens, "::")?;
    let tl = next(tokens).ok_or(ParseError::Empty)?;
    ensure!(
        is_identifier(&tl),
        ParseError::InvalidSyntax(tl.clone()).to_string()
    );
    expect(tokens, "->")?;
    let cons_body = parse_expr(tokens, 0)?;
    Ok(Expression::Match {
        scrutinee: Box::new(scrutinee),
        nil_case: Box::new(nil_case),
        cons_pattern: (hd, tl, Box::new(cons_body)),
    })
}

fn parse_application(tokens: &mut VecDeque<String>) -> Result<Expression> {
    let mut func = parse_atom(tokens)?;
    loop {
        match peek(tokens) {
            Some("then" | "else" | "in" | "|" | "->" | "with") => break,
            Some(op) if precedence(op).is_some() => break,
            None => break,
            _ => {
                if !starts_primary(tokens) {
                    break;
                }
                let arg = parse_atom(tokens)?;
                func = Expression::App {
                    function: Box::new(func),
                    argument: Box::new(arg),
                };
            }
        }
    }
    Ok(func)
}

fn parse_atom(tokens: &mut VecDeque<String>) -> Result<Expression> {
    match next(tokens).ok_or(ParseError::Empty)? {
        t if t.parse::<isize>().is_ok() => Ok(Expression::Integer(t.parse::<isize>().unwrap())),
        t if t == "true" => Ok(Expression::Bool(true)),
        t if t == "false" => Ok(Expression::Bool(false)),
        t if t == "(" => {
            let e = parse_expr(tokens, 0)?;
            match next(tokens) {
                Some(s) if s == ")" => Ok(e),
                Some(s) => bail!(ParseError::Unexpected(s)),
                None => bail!(ParseError::Unclosed),
            }
        }
        t if t == "[]" => Ok(Expression::Nil),
        t if is_identifier(&t) => Ok(Expression::Variable(t)),
        other => bail!(ParseError::Unexpected(other)),
    }
}
