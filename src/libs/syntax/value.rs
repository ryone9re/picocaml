use std::fmt::Display;

use crate::{
    adapter::{RBool, RInteger, Symbol},
    structure::Structure,
    syntax::ast::Expression,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Integer(RInteger),
    Bool(RBool),
    Closure {
        structure: Structure,
        parameter: Symbol,
        body: Expression,
    },
    RecClosure {
        structure: Structure,
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
            Value::Integer(n) => write!(f, "{}", n),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Closure {
                structure: _structure,
                parameter,
                body: _body,
            } => write!(f, "fun {} -> (...)", parameter),
            Value::RecClosure {
                structure: _structure,
                call_name,
                parameter,
                body: _body,
            } => write!(f, "rec {} = fun {} -> (...)", call_name, parameter),
            Value::Nil => write!(f, "[]"),
            Value::Cons { car, cdr } => write!(f, "{} :: {}", car, cdr),
        }
    }
}
