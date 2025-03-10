use std::collections::HashMap;

use crate::{
    ast::Expression,
    eval::Structure,
    types::{RBool, RInteger, Symbol},
};

pub type Environment = HashMap<Symbol, Value>;

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
