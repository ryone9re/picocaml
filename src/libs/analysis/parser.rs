use std::collections::VecDeque;

use anyhow::{Result, bail};
use thiserror::Error;

use crate::syntax::ast::Expression;

#[derive(Debug, Error)]
enum ParseError {
    #[error("Empty")]
    Empty,
    #[error("Invalid syntax: {0}")]
    InvalidSyntax(String),
}

pub fn parse_expression(mut tokens: VecDeque<String>) -> Result<Expression> {
    let token = tokens.pop_front();

    if let Some(token) = token {
        return match token {
            token if token.parse::<isize>().is_ok() => parse_integer_literal(token),
            token if token.parse::<bool>().is_ok() => parse_bool_literal(token),
            _ => bail!(ParseError::InvalidSyntax(token)),
        };
    }

    bail!(ParseError::Empty)
}

fn parse_integer_literal(token: String) -> Result<Expression> {
    if let Ok(integer) = token.parse() {
        return Ok(Expression::Integer(integer));
    }
    bail!(ParseError::InvalidSyntax(token))
}

fn parse_bool_literal(token: String) -> Result<Expression> {
    if let Ok(bool) = token.parse() {
        return Ok(Expression::Bool(bool));
    }
    bail!(ParseError::InvalidSyntax(token))
}
