use crate::{
    adapter::{RBool, RInteger, Symbol},
    ast::Expression,
    eval::Structure,
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
