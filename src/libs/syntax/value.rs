use std::fmt::Display;

use crate::{
    adapter::{RBool, RInteger, Symbol},
    execution::environment::Environment,
    syntax::ast::Expression,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Integer(RInteger),
    Bool(RBool),
    Closure {
        environment: Environment,
        parameter: Symbol,
        body: Expression,
    },
    RecClosure {
        environment: Environment,
        call_name: Symbol,
        parameter: Symbol,
        body: Expression,
    },
    Nil,
    Cons {
        car: Box<Value>,
        cdr: Box<Value>,
    },
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Integer(i) => write!(f, "{}", i),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Closure {
                parameter, body, ..
            } => write!(f, "<fun {} -> {}>", parameter, body),
            Value::RecClosure {
                call_name,
                parameter,
                body,
                ..
            } => write!(f, "<recfun {} {} -> {}>", call_name, parameter, body),
            Value::Nil => write!(f, "nil"),
            Value::Cons { car, cdr } => write!(f, "(cons {} {})", car, cdr),
        }
    }
}
