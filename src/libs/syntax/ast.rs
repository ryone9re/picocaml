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
        match self {
            Expression::Integer(i) => write!(f, "{}", i),
            Expression::Bool(b) => write!(f, "{}", b),
            Expression::Variable(sym) => write!(f, "{}", sym),
            Expression::Plus {
                expression1,
                expression2,
            } => write!(f, "(+ {} {})", expression1, expression2),
            Expression::Minus {
                expression1,
                expression2,
            } => write!(f, "(- {} {})", expression1, expression2),
            Expression::Times {
                expression1,
                expression2,
            } => write!(f, "(* {} {})", expression1, expression2),
            Expression::LessThan {
                expression1,
                expression2,
            } => write!(f, "(< {} {})", expression1, expression2),
            Expression::If {
                predicate,
                consequent,
                alternative,
            } => write!(f, "(if {} {} {})", predicate, consequent, alternative),
            Expression::Let {
                variable,
                bound,
                body,
            } => write!(f, "(let ({} {}) {})", variable, bound, body),
            Expression::Fun { parameter, body } => write!(f, "(fun {} {})", parameter, body),
            Expression::App { function, argument } => write!(f, "(app {} {})", function, argument),
            Expression::LetRec {
                variable,
                bound_function,
                body,
            } => write!(f, "(letrec ({} {}) {})", variable, bound_function, body),
            Expression::Nil => write!(f, "nil"),
            Expression::Cons { car, cdr } => write!(f, "(cons {} {})", car, cdr),
            Expression::Match {
                scrutinee,
                nil_case,
                cons_pattern,
            } => {
                let (car, cdr, cons_body) = cons_pattern;
                write!(
                    f,
                    "(match {} (nil {}) (cons ({} {}) {}))",
                    scrutinee, nil_case, car, cdr, cons_body
                )
            }
        }
    }
}
