use std::fmt::Display;

use crate::adapter::{RBool, RInteger, Symbol};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    Integer(RInteger),
    Bool(RBool),
    Variable(Symbol),
    Plus {
        expression1: Box<Expression>,
        expression2: Box<Expression>,
    },
    Minus {
        expression1: Box<Expression>,
        expression2: Box<Expression>,
    },
    Times {
        expression1: Box<Expression>,
        expression2: Box<Expression>,
    },
    LessThan {
        expression1: Box<Expression>,
        expression2: Box<Expression>,
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
        bound_function: Box<Expression>, // Expressionを評価した結果が再帰関数であることを暗黙的に前提とする
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

impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}
