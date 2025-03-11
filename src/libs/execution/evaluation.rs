use anyhow::{Result, anyhow, bail};
use thiserror::Error;

use crate::{
    adapter::{
        RArithmeticOp, RBool, RComparisonOp, RInteger, Symbol, r_lt, r_minus, r_plus, r_times,
    },
    structure::Structure,
    syntax::{ast::Expression, value::Value},
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

pub fn eval(structure: Structure, expression: Expression) -> Result<(Structure, Value)> {
    let (new_structure, value) = match expression {
        Expression::Integer(n) => eval_integer(structure, n)?,
        Expression::Bool(b) => eval_bool(structure, b)?,
        Expression::Variable(variable) => eval_variable(structure, variable)?,
        Expression::Plus { e1, e2 } => eval_arithmetic_op(structure, *e1, *e2, r_plus)?,
        Expression::Minus { e1, e2 } => eval_arithmetic_op(structure, *e1, *e2, r_minus)?,
        Expression::Times { e1, e2 } => eval_arithmetic_op(structure, *e1, *e2, r_times)?,
        Expression::LessThan { e1, e2 } => eval_comparison_op(structure, *e1, *e2, r_lt)?,
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
            cons_pattern: (car, cdr, cons_case),
        } => eval_match(structure, *scrutinee, *nil_case, (car, cdr, *cons_case))?,
    };

    Ok((new_structure, value))
}

fn eval_integer(structure: Structure, n: RInteger) -> Result<(Structure, Value)> {
    Ok((structure, Value::Integer(n)))
}

fn eval_bool(structure: Structure, b: RBool) -> Result<(Structure, Value)> {
    Ok((structure, Value::Bool(b)))
}

fn eval_variable(structure: Structure, variable: Symbol) -> Result<(Structure, Value)> {
    let value = structure
        .get_variable_value(&variable)
        .ok_or(anyhow!(EvalError::UndefinedVariable(variable.clone())))?;

    Ok((structure, value))
}

fn eval_arithmetic_op(
    structure: Structure,
    e1: Expression,
    e2: Expression,
    op: RArithmeticOp,
) -> Result<(Structure, Value)> {
    let (_, e1) = eval(structure.clone(), e1)?;
    let (_, e2) = eval(structure.clone(), e2)?;

    if let (Value::Integer(e1_value), Value::Integer(e2_value)) = (e1, e2) {
        return Ok((structure, Value::Integer(op(e1_value, e2_value))));
    }

    bail!(EvalError::InvalidExpression)
}

fn eval_comparison_op(
    structure: Structure,
    e1: Expression,
    e2: Expression,
    op: RComparisonOp,
) -> Result<(Structure, Value)> {
    let (_, e1) = eval(structure.clone(), e1)?;
    let (_, e2) = eval(structure.clone(), e2)?;

    if let (Value::Integer(e1_value), Value::Integer(e2_value)) = (e1, e2) {
        return Ok((structure, Value::Bool(op(e1_value, e2_value))));
    }

    bail!(EvalError::InvalidExpression)
}

fn eval_if(
    structure: Structure,
    predicate: Expression,
    consequent: Expression,
    alternative: Expression,
) -> Result<(Structure, Value)> {
    let (_, predicate) = eval(structure.clone(), predicate)?;

    match predicate {
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
) -> Result<(Structure, Value)> {
    let (_, bound) = eval(structure.clone(), bound)?;
    let new_structure = structure.bind_variable(variable, bound)?;

    eval(new_structure, body)
}

fn eval_fun(
    structure: Structure,
    parameter: Symbol,
    body: Expression,
) -> Result<(Structure, Value)> {
    let captured_structure = structure.clone();

    Ok((
        structure,
        Value::Closure {
            structure: captured_structure,
            parameter,
            body,
        },
    ))
}

fn eval_app(
    structure: Structure,
    function: Expression,
    argument: Expression,
) -> Result<(Structure, Value)> {
    let (_, closure) = eval(structure.clone(), function)?;
    let (_, argument) = eval(structure.clone(), argument)?;

    match closure {
        Value::Closure {
            structure,
            parameter,
            body,
        } => {
            let captured_structure = structure.bind_variable(parameter, argument)?;

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
            let structure = structure.bind_variable(call_name, rec_closure)?;
            let captured_structure = structure.bind_variable(parameter, argument)?;

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
) -> Result<(Structure, Value)> {
    if let Expression::Fun {
        parameter,
        body: function_body,
    } = bound_function
    {
        let captured_structure = structure.clone();
        let call_name = variable.clone();
        let structure = structure.bind_variable(
            variable,
            Value::RecClosure {
                structure: captured_structure,
                call_name,
                parameter,
                body: *function_body,
            },
        )?;

        return eval(structure, body);
    }

    eval(structure, body)
}

fn eval_nil(structure: Structure) -> Result<(Structure, Value)> {
    Ok((structure, Value::Nil))
}

fn eval_cons(structure: Structure, car: Expression, cdr: Expression) -> Result<(Structure, Value)> {
    let (_, car) = eval(structure.clone(), car)?;
    let (_, cdr) = eval(structure.clone(), cdr)?;

    Ok((
        structure,
        Value::Cons {
            car: car.into(),
            cdr: cdr.into(),
        },
    ))
}

fn eval_match(
    structure: Structure,
    scrutinee: Expression,
    nil_case: Expression,
    cons_pattern: (Symbol, Symbol, Expression),
) -> Result<(Structure, Value)> {
    let (_, pattern) = eval(structure.clone(), scrutinee)?;

    match pattern {
        Value::Nil => eval(structure, nil_case),
        Value::Cons { car, cdr } => {
            let (car_variable, cdr_variable, cons_case) = cons_pattern;
            let structure = structure
                .bind_variable(car_variable, *car)?
                .bind_variable(cdr_variable, *cdr)?;

            eval(structure, cons_case)
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
            e1: Expression::Integer(3).into(),
            e2: Expression::Times {
                e1: Expression::Integer(5).into(),
                e2: Expression::Integer(2).into(),
            }
            .into(),
        };

        let result = eval(Structure::default(), expr);

        assert!(result.is_ok());
        let value = result.unwrap().1;
        assert!(matches!(value, Value::Integer(13)));
    }

    #[test]
    fn test_let_binding() {
        // let x = 10 in x + 5
        let expr = Expression::Let {
            variable: "x".to_string(),
            bound: Expression::Integer(10).into(),
            body: Expression::Plus {
                e1: Expression::Variable("x".to_string()).into(),
                e2: Expression::Integer(5).into(),
            }
            .into(),
        };

        let result = eval(Structure::default(), expr);

        assert!(result.is_ok());
        let value = result.unwrap().1;
        assert!(matches!(value, Value::Integer(15)));
    }

    #[test]
    fn test_if_expression() {
        // if 5 < 10 then 20 else 30
        let expr = Expression::If {
            predicate: Expression::LessThan {
                e1: Expression::Integer(5).into(),
                e2: Expression::Integer(10).into(),
            }
            .into(),
            consequent: Expression::Integer(20).into(),
            alternative: Expression::Integer(30).into(),
        };

        let result = eval(Structure::default(), expr);

        assert!(result.is_ok());
        let value = result.unwrap().1;
        assert!(matches!(value, Value::Integer(20)));
    }

    #[test]
    fn test_function_application() {
        // (fun x -> x + 1) 5
        let expr = Expression::App {
            function: Expression::Fun {
                parameter: "x".to_string(),
                body: Expression::Plus {
                    e1: Expression::Variable("x".to_string()).into(),
                    e2: Expression::Integer(1).into(),
                }
                .into(),
            }
            .into(),
            argument: Expression::Integer(5).into(),
        };

        let result = eval(Structure::default(), expr);

        assert!(result.is_ok());
        let value = result.unwrap().1;
        assert!(matches!(value, Value::Integer(6)));
    }

    #[test]
    fn test_recursive_function() {
        // let rec fact = fun n -> if n < 2 then 1 else n * fact (n - 1) in fact 5
        let expr = Expression::LetRec {
            variable: "fact".to_string(),
            bound_function: Expression::Fun {
                parameter: "n".to_string(),
                body: Expression::If {
                    predicate: Expression::LessThan {
                        e1: Expression::Variable("n".to_string()).into(),
                        e2: Expression::Integer(2).into(),
                    }
                    .into(),
                    consequent: Expression::Integer(1).into(),
                    alternative: Expression::Times {
                        e1: Expression::Variable("n".to_string()).into(),
                        e2: Expression::App {
                            function: Expression::Variable("fact".to_string()).into(),
                            argument: Expression::Minus {
                                e1: Expression::Variable("n".to_string()).into(),
                                e2: Expression::Integer(1).into(),
                            }
                            .into(),
                        }
                        .into(),
                    }
                    .into(),
                }
                .into(),
            }
            .into(),
            body: Expression::App {
                function: Expression::Variable("fact".to_string()).into(),
                argument: Expression::Integer(5).into(),
            }
            .into(),
        };

        let result = eval(Structure::default(), expr);

        assert!(result.is_ok());
        let value = result.unwrap().1;
        assert!(matches!(value, Value::Integer(120)));
    }

    #[test]
    fn test_list_operations() {
        // match 1::2::[] with [] -> 0 | hd::tl -> hd
        let expr = Expression::Match {
            scrutinee: Expression::Cons {
                car: Expression::Integer(1).into(),
                cdr: Expression::Cons {
                    car: Expression::Integer(2).into(),
                    cdr: Expression::Nil.into(),
                }
                .into(),
            }
            .into(),
            nil_case: Expression::Integer(0).into(),
            cons_pattern: (
                "hd".to_string(),
                "tl".to_string(),
                Expression::Variable("hd".to_string()).into(),
            ),
        };

        let result = eval(Structure::default(), expr);

        assert!(result.is_ok());
        let value = result.unwrap().1;
        assert!(matches!(value, Value::Integer(1)));
    }
}
