use crate::{
    adapter::{RBool, RInteger, Variable},
    ast::Expression,
    eval::Environment,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Integer(RInteger),
    Bool(RBool),
    Closure {
        variable: Variable,
        body: Box<Expression>,
        environment: Environment,
    },
    RecClosure,
    Nil,
    Cons {
        car: Box<Value>,
        cdr: Box<Value>,
    },
}
