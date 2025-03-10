use std::{
    collections::HashMap,
    ops::{Add, Mul, Sub},
};

use anyhow::{Result, anyhow, bail};
use thiserror::Error;

use crate::{
    adapter::{RBool, RInteger, Symbol},
    ast::Expression,
    value::Value,
};

#[derive(Debug, Error)]
enum EvalError {
    #[error("Invalid expression")]
    InvalidExpression,
    #[error("Undefined variable: {0}")]
    UndefinedVariable(Symbol),
    #[error("Type error!")]
    TypeError,
}

type Environment = HashMap<Symbol, Box<Value>>;

type TypeEnvironment = HashMap<Symbol, ()>;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Structure {
    environment: Environment,
    type_environment: TypeEnvironment,
}

impl Structure {
    fn assign_variable(self, variable: Symbol, value: Box<Value>) -> Result<Structure> {
        let mut new_structure = self.clone();
        new_structure.environment.insert(variable, value);
        Ok(new_structure)
    }
}

pub fn eval(structure: Structure, expression: Expression) -> Result<(Structure, Box<Value>)> {
    dbg!(&structure);
    dbg!(&expression);
    let (new_structure, value) = match expression {
        Expression::Integer(n) => eval_integer(structure, n)?,
        Expression::Bool(b) => eval_bool(structure, b)?,
        Expression::Variable(variable) => eval_variable(structure, variable)?,
        Expression::Plus { e1, e2 } => eval_plus(structure, *e1, *e2)?,
        Expression::Minus { e1, e2 } => eval_minus(structure, *e1, *e2)?,
        Expression::Times { e1, e2 } => eval_times(structure, *e1, *e2)?,
        Expression::LessThan { e1, e2 } => eval_lt(structure, *e1, *e2)?,
        Expression::If {
            predicate,
            consequent,
            alternative,
        } => eval_if(structure, *predicate, *consequent, *alternative)?,
        Expression::Let {
            variable,
            bound,
            body,
        } => eval_let(structure, variable, *bound, *body)?,
        Expression::Fun { parameter, body } => eval_fun(structure, parameter, *body)?,
        Expression::App { function, argument } => eval_app(structure, *function, *argument)?,
        Expression::LetRec {
            variable,
            bound_function,
            body,
        } => eval_let_rec(structure, variable, *bound_function, *body)?,
        Expression::Nil => eval_nil(structure)?,
        Expression::Cons { car, cdr } => eval_cons(structure, *car, *cdr)?,
        Expression::Match {
            scrutinee,
            nil_case,
            cons_case,
        } => eval_match(structure, *scrutinee, *nil_case, cons_case)?,
    };

    Ok((new_structure, value))
}

fn eval_integer(structure: Structure, n: RInteger) -> Result<(Structure, Box<Value>)> {
    Ok((structure, Value::Integer(n).into()))
}

fn eval_bool(structure: Structure, b: RBool) -> Result<(Structure, Box<Value>)> {
    Ok((structure, Value::Bool(b).into()))
}

fn eval_variable(structure: Structure, variable: Symbol) -> Result<(Structure, Box<Value>)> {
    let value = structure
        .environment
        .get(&variable)
        .ok_or(anyhow!(EvalError::UndefinedVariable(variable.clone())))?
        .clone();

    Ok((structure, value))
}

fn eval_plus(
    structure: Structure,
    e1: Expression,
    e2: Expression,
) -> Result<(Structure, Box<Value>)> {
    let (_, e1) = eval(structure.clone(), e1)?;
    let (_, e2) = eval(structure.clone(), e2)?;

    if let (Value::Integer(e1_value), Value::Integer(e2_value)) = (*e1, *e2) {
        return Ok((structure, Value::Integer(e1_value.add(e2_value)).into()));
    }

    bail!(EvalError::InvalidExpression)
}

fn eval_minus(
    structure: Structure,
    e1: Expression,
    e2: Expression,
) -> Result<(Structure, Box<Value>)> {
    let (_, e1) = eval(structure.clone(), e1)?;
    let (_, e2) = eval(structure.clone(), e2)?;

    if let (Value::Integer(e1_value), Value::Integer(e2_value)) = (*e1, *e2) {
        return Ok((structure, Value::Integer(e1_value.sub(e2_value)).into()));
    }

    bail!(EvalError::InvalidExpression)
}

fn eval_times(
    structure: Structure,
    e1: Expression,
    e2: Expression,
) -> Result<(Structure, Box<Value>)> {
    let (_, e1) = eval(structure.clone(), e1)?;
    let (_, e2) = eval(structure.clone(), e2)?;

    if let (Value::Integer(e1_value), Value::Integer(e2_value)) = (*e1, *e2) {
        return Ok((structure, Value::Integer(e1_value.mul(e2_value)).into()));
    }

    bail!(EvalError::InvalidExpression)
}

fn eval_lt(
    structure: Structure,
    e1: Expression,
    e2: Expression,
) -> Result<(Structure, Box<Value>)> {
    let (_, e1) = eval(structure.clone(), e1)?;
    let (_, e2) = eval(structure.clone(), e2)?;

    if let (Value::Integer(e1_value), Value::Integer(e2_value)) = (*e1, *e2) {
        return Ok((structure, Value::Bool(e1_value.lt(&e2_value)).into()));
    }

    bail!(EvalError::InvalidExpression)
}

fn eval_if(
    structure: Structure,
    predicate: Expression,
    consequent: Expression,
    alternative: Expression,
) -> Result<(Structure, Box<Value>)> {
    let (_, predicate) = eval(structure.clone(), predicate)?;

    match *predicate {
        Value::Bool(b) if b => eval(structure, consequent),
        Value::Bool(b) if !b => eval(structure, alternative),
        _ => bail!(EvalError::TypeError),
    }
}

fn eval_let(
    structure: Structure,
    variable: Symbol,
    bound: Expression,
    body: Expression,
) -> Result<(Structure, Box<Value>)> {
    let (_, bound) = eval(structure.clone(), bound)?;
    let new_structure = structure.assign_variable(variable, bound)?;

    eval(new_structure, body)
}

fn eval_fun(
    structure: Structure,
    parameter: Symbol,
    body: Expression,
) -> Result<(Structure, Box<Value>)> {
    let captured_structure = structure.clone();

    Ok((
        structure,
        Value::Closure {
            structure: captured_structure,
            parameter,
            body,
        }
        .into(),
    ))
}

fn eval_app(
    structure: Structure,
    function: Expression,
    argument: Expression,
) -> Result<(Structure, Box<Value>)> {
    let (_, closure) = eval(structure.clone(), function)?;
    let (_, argument) = eval(structure.clone(), argument)?;

    match *closure {
        Value::Closure {
            structure,
            parameter,
            body,
        } => {
            let captured_structure = structure.assign_variable(parameter, argument)?;

            eval(captured_structure, body)
        }
        Value::RecClosure {
            structure,
            call_name,
            parameter,
            body,
        } => {
            let rec_closure = Value::RecClosure {
                structure: structure.clone(),
                call_name: call_name.clone(),
                parameter: parameter.clone(),
                body: body.clone(),
            };
            let structure = structure.assign_variable(call_name, rec_closure.into())?;
            let captured_structure = structure.assign_variable(parameter, argument)?;

            eval(captured_structure, body)
        }
        _ => bail!(EvalError::InvalidExpression),
    }
}

fn eval_let_rec(
    structure: Structure,
    variable: Symbol,
    bound_function: Expression,
    body: Expression,
) -> Result<(Structure, Box<Value>)> {
    if let Expression::Fun {
        parameter,
        body: function_body,
    } = bound_function
    {
        let captured_structure = structure.clone();
        let call_name = variable.clone();
        let structure = structure.assign_variable(
            variable,
            Value::RecClosure {
                structure: captured_structure,
                call_name,
                parameter,
                body: *function_body,
            }
            .into(),
        )?;

        return eval(structure, body);
    }

    eval(structure, body)
}

fn eval_nil(structure: Structure) -> Result<(Structure, Box<Value>)> {
    Ok((structure, Value::Nil.into()))
}

fn eval_cons(
    structure: Structure,
    car: Expression,
    cdr: Expression,
) -> Result<(Structure, Box<Value>)> {
    let (_, car) = eval(structure.clone(), car)?;
    let (_, cdr) = eval(structure.clone(), cdr)?;

    Ok((structure, Value::Cons { car, cdr }.into()))
}

fn eval_match(
    structure: Structure,
    scrutinee: Expression,
    nil_case: Expression,
    cons_case: (Symbol, Symbol, Box<Expression>),
) -> Result<(Structure, Box<Value>)> {
    let (_, pattern) = eval(structure.clone(), scrutinee)?;

    match *pattern {
        Value::Nil => eval(structure, nil_case),
        Value::Cons { car, cdr } => {
            let (car_variable, cdr_variable, expression) = cons_case;
            let structure = structure
                .assign_variable(car_variable, car)?
                .assign_variable(cdr_variable, cdr)?;

            eval(structure, *expression)
        }
        _ => bail!(EvalError::InvalidExpression),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_arithmetic() {
        // 3 + 5 * 2
        let expr = Expression::Plus {
            e1: Box::new(Expression::Integer(3)),
            e2: Box::new(Expression::Times {
                e1: Box::new(Expression::Integer(5)),
                e2: Box::new(Expression::Integer(2)),
            }),
        };

        let result = eval(Structure::default(), expr);

        assert!(result.is_ok());
        let value = result.unwrap().1;
        assert!(matches!(*value, Value::Integer(13)));
    }

    #[test]
    fn test_let_binding() {
        // let x = 10 in x + 5
        let expr = Expression::Let {
            variable: "x".to_string(),
            bound: Box::new(Expression::Integer(10)),
            body: Box::new(Expression::Plus {
                e1: Box::new(Expression::Variable("x".to_string())),
                e2: Box::new(Expression::Integer(5)),
            }),
        };

        let result = eval(Structure::default(), expr);

        assert!(result.is_ok());
        let value = result.unwrap().1;
        assert!(matches!(*value, Value::Integer(15)));
    }

    #[test]
    fn test_if_expression() {
        // if 5 < 10 then 20 else 30
        let expr = Expression::If {
            predicate: Box::new(Expression::LessThan {
                e1: Box::new(Expression::Integer(5)),
                e2: Box::new(Expression::Integer(10)),
            }),
            consequent: Box::new(Expression::Integer(20)),
            alternative: Box::new(Expression::Integer(30)),
        };

        let result = eval(Structure::default(), expr);

        assert!(result.is_ok());
        let value = result.unwrap().1;
        assert!(matches!(*value, Value::Integer(20)));
    }

    #[test]
    fn test_function_application() {
        // (fun x -> x + 1) 5
        let expr = Expression::App {
            function: Box::new(Expression::Fun {
                parameter: "x".to_string(),
                body: Box::new(Expression::Plus {
                    e1: Box::new(Expression::Variable("x".to_string())),
                    e2: Box::new(Expression::Integer(1)),
                }),
            }),
            argument: Box::new(Expression::Integer(5)),
        };

        let result = eval(Structure::default(), expr);

        assert!(result.is_ok());
        let value = result.unwrap().1;
        assert!(matches!(*value, Value::Integer(6)));
    }

    #[test]
    fn test_recursive_function() {
        // let rec fact = fun n -> if n < 2 then 1 else n * fact (n - 1) in fact 5
        let expr = Expression::LetRec {
            variable: "fact".to_string(),
            bound_function: Box::new(Expression::Fun {
                parameter: "n".to_string(),
                body: Box::new(Expression::If {
                    predicate: Box::new(Expression::LessThan {
                        e1: Box::new(Expression::Variable("n".to_string())),
                        e2: Box::new(Expression::Integer(2)),
                    }),
                    consequent: Box::new(Expression::Integer(1)),
                    alternative: Box::new(Expression::Times {
                        e1: Box::new(Expression::Variable("n".to_string())),
                        e2: Box::new(Expression::App {
                            function: Box::new(Expression::Variable("fact".to_string())),
                            argument: Box::new(Expression::Minus {
                                e1: Box::new(Expression::Variable("n".to_string())),
                                e2: Box::new(Expression::Integer(1)),
                            }),
                        }),
                    }),
                }),
            }),
            body: Box::new(Expression::App {
                function: Box::new(Expression::Variable("fact".to_string())),
                argument: Box::new(Expression::Integer(5)),
            }),
        };

        let result = eval(Structure::default(), expr);

        assert!(result.is_ok());
        let value = result.unwrap().1;
        assert!(matches!(*value, Value::Integer(120)));
    }

    #[test]
    fn test_list_operations() {
        // match 1::2::[] with [] -> 0 | hd::tl -> hd
        let expr = Expression::Match {
            scrutinee: Box::new(Expression::Cons {
                car: Box::new(Expression::Integer(1)),
                cdr: Box::new(Expression::Cons {
                    car: Box::new(Expression::Integer(2)),
                    cdr: Box::new(Expression::Nil),
                }),
            }),
            nil_case: Box::new(Expression::Integer(0)),
            cons_case: (
                "hd".to_string(),
                "tl".to_string(),
                Box::new(Expression::Variable("hd".to_string())),
            ),
        };

        let result = eval(Structure::default(), expr);

        assert!(result.is_ok());
        let value = result.unwrap().1;
        assert!(matches!(*value, Value::Integer(1)));
    }
}
