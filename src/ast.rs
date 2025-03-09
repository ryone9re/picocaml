use crate::adapter::{RBool, RInteger, Variable};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    Integer(RInteger),
    Bool(RBool),
    Variable(Variable),
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
        variable: Variable,
        bound: Box<Expression>,
        body: Box<Expression>,
    },
    Fun {
        variable: Variable,
        body: Box<Expression>,
    },
    App {
        function: Box<Expression>,
        argument: Box<Expression>,
    },
    LetRec {
        variable: Variable,
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
        cons_case: Option<(Variable, Variable, Box<Expression>)>,
    },
}
