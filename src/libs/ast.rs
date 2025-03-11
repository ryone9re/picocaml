use crate::adapter::{RBool, RInteger, Symbol};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    Integer(RInteger),
    Bool(RBool),
    Variable(Symbol),
    Plus {
        e1: Box<Expression>,
        e2: Box<Expression>,
    },
    Minus {
        e1: Box<Expression>,
        e2: Box<Expression>,
    },
    Times {
        e1: Box<Expression>,
        e2: Box<Expression>,
    },
    LessThan {
        e1: Box<Expression>,
        e2: Box<Expression>,
    },
    If {
        predicate: Box<Expression>,
        consequent: Box<Expression>,
        alternative: Box<Expression>,
    },
    Let {
        variable: Symbol,
        bound: Box<Expression>,
        body: Box<Expression>,
    },
    Fun {
        parameter: Symbol,
        body: Box<Expression>,
    },
    App {
        function: Box<Expression>,
        argument: Box<Expression>,
    },
    LetRec {
        variable: Symbol,
        bound_function: Box<Expression>,
        body: Box<Expression>,
    },
    Nil,
    Cons {
        car: Box<Expression>,
        cdr: Box<Expression>,
    },
    Match {
        scrutinee: Box<Expression>,
        nil_case: Box<Expression>,
        cons_pattern: (Symbol, Symbol, Box<Expression>),
    },
}
