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
        write!(f, "{:?}", self)
    }
}
