use anyhow::{Result, anyhow, bail};
use thiserror::Error;

use crate::{
    adapter::{
        RArithmeticOperation, RBool, RComparisonOperation, RInteger, Symbol, r_lt, r_minus, r_plus,
        r_times,
    },
    execution::environment::Environment,
    syntax::{ast::Expression, value::Value},
};

#[derive(Debug, Error)]
enum EvalError {
    #[error("Invalid expression")]
    InvalidExpression,
    #[error("Undefined variable: {0}")]
    UndefinedVariable(Symbol),
}

pub fn eval(environment: Environment, expression: Expression) -> Result<(Environment, Value)> {
    match expression {
        Expression::Integer(n) => eval_integer(environment, n),
        Expression::Bool(b) => eval_bool(environment, b),
        Expression::Variable(variable) => eval_variable(environment, variable),
        Expression::Plus {
            expression1,
            expression2,
        } => eval_arithmetic_operation(environment, *expression1, *expression2, r_plus),
        Expression::Minus {
            expression1,
            expression2,
        } => eval_arithmetic_operation(environment, *expression1, *expression2, r_minus),
        Expression::Times {
            expression1,
            expression2,
        } => eval_arithmetic_operation(environment, *expression1, *expression2, r_times),
        Expression::LessThan {
            expression1,
            expression2,
        } => eval_comparison_operation(environment, *expression1, *expression2, r_lt),
        Expression::If {
            predicate,
            consequent,
            alternative,
        } => eval_if(environment, *predicate, *consequent, *alternative),
        Expression::Let {
            variable,
            bound,
            body,
        } => eval_let(environment, variable, *bound, *body),
        Expression::Fun { parameter, body } => eval_fun(environment, parameter, *body),
        Expression::App { function, argument } => eval_app(environment, *function, *argument),
        Expression::LetRec {
            variable,
            bound_function,
            body,
        } => eval_let_rec(environment, variable, *bound_function, *body),
        Expression::Nil => eval_nil(environment),
        Expression::Cons { car, cdr } => eval_cons(environment, *car, *cdr),
        Expression::Match {
            scrutinee,
            nil_case,
            cons_pattern: (car, cdr, cons_case),
        } => eval_match(environment, *scrutinee, *nil_case, (car, cdr, *cons_case)),
    }
}

fn eval_integer(environment: Environment, n: RInteger) -> Result<(Environment, Value)> {
    Ok((environment, Value::Integer(n)))
}

fn eval_bool(environment: Environment, b: RBool) -> Result<(Environment, Value)> {
    Ok((environment, Value::Bool(b)))
}

fn eval_variable(environment: Environment, variable: Symbol) -> Result<(Environment, Value)> {
    let value = environment
        .get(&variable)
        .ok_or(anyhow!(EvalError::UndefinedVariable(variable.clone())))?;

    Ok((environment, value))
}

fn eval_arithmetic_operation(
    environment: Environment,
    expression1: Expression,
    expression2: Expression,
    operation: RArithmeticOperation,
) -> Result<(Environment, Value)> {
    let (_, expression1) = eval(environment.clone(), expression1)?;
    let (_, expression2) = eval(environment.clone(), expression2)?;

    match (expression1, expression2) {
        (Value::Integer(expression1_value), Value::Integer(expression2_value)) => Ok((
            environment,
            Value::Integer(operation(expression1_value, expression2_value)),
        )),
        _ => bail!(EvalError::InvalidExpression),
    }
}

fn eval_comparison_operation(
    environment: Environment,
    expression1: Expression,
    expression2: Expression,
    operation: RComparisonOperation,
) -> Result<(Environment, Value)> {
    let (_, expression1) = eval(environment.clone(), expression1)?;
    let (_, expression2) = eval(environment.clone(), expression2)?;

    match (expression1, expression2) {
        (Value::Integer(expression1_value), Value::Integer(expression2_value)) => Ok((
            environment,
            Value::Bool(operation(expression1_value, expression2_value)),
        )),
        _ => bail!(EvalError::InvalidExpression),
    }
}

fn eval_if(
    environment: Environment,
    predicate: Expression,
    consequent: Expression,
    alternative: Expression,
) -> Result<(Environment, Value)> {
    let (_, predicate) = eval(environment.clone(), predicate)?;

    match predicate {
        Value::Bool(b) if b => eval(environment, consequent),
        Value::Bool(b) if !b => eval(environment, alternative),
        _ => bail!(EvalError::InvalidExpression),
    }
}

fn eval_let(
    environment: Environment,
    variable: Symbol,
    bound: Expression,
    body: Expression,
) -> Result<(Environment, Value)> {
    let (_, bound) = eval(environment.clone(), bound)?;
    let new_environment = environment.bind(variable, bound)?;

    eval(new_environment, body)
}

fn eval_fun(
    environment: Environment,
    parameter: Symbol,
    body: Expression,
) -> Result<(Environment, Value)> {
    let captured_environment = environment.clone();

    Ok((
        environment,
        Value::Closure {
            environment: captured_environment,
            parameter,
            body,
        },
    ))
}

fn eval_app(
    environment: Environment,
    function: Expression,
    argument: Expression,
) -> Result<(Environment, Value)> {
    let (_, closure) = eval(environment.clone(), function)?;
    let (_, argument) = eval(environment.clone(), argument)?;

    match closure {
        Value::Closure {
            environment,
            parameter,
            body,
        } => {
            let captured_environment = environment.bind(parameter, argument)?;

            eval(captured_environment, body)
        }
        Value::RecClosure {
            environment,
            call_name,
            parameter,
            body,
        } => {
            let rec_closure = Value::RecClosure {
                environment: environment.clone(),
                call_name: call_name.clone(),
                parameter: parameter.clone(),
                body: body.clone(),
            };
            let environment = environment.bind(call_name, rec_closure)?;
            let captured_environment = environment.bind(parameter, argument)?;

            eval(captured_environment, body)
        }
        _ => bail!(EvalError::InvalidExpression),
    }
}

fn eval_let_rec(
    environment: Environment,
    variable: Symbol,
    bound_function: Expression,
    body: Expression,
) -> Result<(Environment, Value)> {
    if let Expression::Fun {
        parameter,
        body: function_body,
    } = bound_function
    {
        let captured_environment = environment.clone();
        let call_name = variable.clone();
        let environment = environment.bind(
            variable,
            Value::RecClosure {
                environment: captured_environment,
                call_name,
                parameter,
                body: *function_body,
            },
        )?;

        return eval(environment, body);
    }

    eval(environment, body)
}

fn eval_nil(environment: Environment) -> Result<(Environment, Value)> {
    Ok((environment, Value::Nil))
}

fn eval_cons(
    environment: Environment,
    car: Expression,
    cdr: Expression,
) -> Result<(Environment, Value)> {
    let (_, car) = eval(environment.clone(), car)?;
    let (_, cdr) = eval(environment.clone(), cdr)?;

    Ok((
        environment,
        Value::Cons {
            car: car.into(),
            cdr: cdr.into(),
        },
    ))
}

fn eval_match(
    environment: Environment,
    scrutinee: Expression,
    nil_case: Expression,
    cons_pattern: (Symbol, Symbol, Expression),
) -> Result<(Environment, Value)> {
    let (_, pattern) = eval(environment.clone(), scrutinee)?;

    match pattern {
        Value::Nil => eval(environment, nil_case),
        Value::Cons { car, cdr } => {
            let (car_variable, cdr_variable, cons_case) = cons_pattern;
            let environment = environment
                .bind(car_variable, *car)?
                .bind(cdr_variable, *cdr)?;

            eval(environment, cons_case)
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
            expression1: Expression::Integer(3).into(),
            expression2: Expression::Times {
                expression1: Expression::Integer(5).into(),
                expression2: Expression::Integer(2).into(),
            }
            .into(),
        };

        let result = eval(Environment::default(), expr);

        assert!(result.is_ok());
        let (_, value) = result.unwrap();
        assert!(matches!(value, Value::Integer(13)));
    }

    #[test]
    fn test_let_binding() {
        // let x = 10 in x + 5
        let expr = Expression::Let {
            variable: "x".to_string(),
            bound: Expression::Integer(10).into(),
            body: Expression::Plus {
                expression1: Expression::Variable("x".to_string()).into(),
                expression2: Expression::Integer(5).into(),
            }
            .into(),
        };

        let result = eval(Environment::default(), expr);

        assert!(result.is_ok());
        let (_, value) = result.unwrap();
        assert!(matches!(value, Value::Integer(15)));
    }

    #[test]
    fn test_if_expression() {
        // if 5 < 10 then 20 else 30
        let expr = Expression::If {
            predicate: Expression::LessThan {
                expression1: Expression::Integer(5).into(),
                expression2: Expression::Integer(10).into(),
            }
            .into(),
            consequent: Expression::Integer(20).into(),
            alternative: Expression::Integer(30).into(),
        };

        let result = eval(Environment::default(), expr);

        assert!(result.is_ok());
        let (_, value) = result.unwrap();
        assert!(matches!(value, Value::Integer(20)));
    }

    #[test]
    fn test_function_application() {
        // (fun x -> x + 1) 5
        let expr = Expression::App {
            function: Expression::Fun {
                parameter: "x".to_string(),
                body: Expression::Plus {
                    expression1: Expression::Variable("x".to_string()).into(),
                    expression2: Expression::Integer(1).into(),
                }
                .into(),
            }
            .into(),
            argument: Expression::Integer(5).into(),
        };

        let result = eval(Environment::default(), expr);

        assert!(result.is_ok());
        let (_, value) = result.unwrap();
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
                        expression1: Expression::Variable("n".to_string()).into(),
                        expression2: Expression::Integer(2).into(),
                    }
                    .into(),
                    consequent: Expression::Integer(1).into(),
                    alternative: Expression::Times {
                        expression1: Expression::Variable("n".to_string()).into(),
                        expression2: Expression::App {
                            function: Expression::Variable("fact".to_string()).into(),
                            argument: Expression::Minus {
                                expression1: Expression::Variable("n".to_string()).into(),
                                expression2: Expression::Integer(1).into(),
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

        let result = eval(Environment::default(), expr);

        assert!(result.is_ok());
        let (_, value) = result.unwrap();
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

        let result = eval(Environment::default(), expr);

        assert!(result.is_ok());
        let (_, value) = result.unwrap();
        assert!(matches!(value, Value::Integer(1)));
    }
}
