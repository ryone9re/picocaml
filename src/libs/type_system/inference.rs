use std::collections::HashSet;

use crate::{
    adapter::unique_simbol,
    type_system::{
        type_environment::TypeEnvironment,
        types::{BaseType, Type},
    },
};
use anyhow::{Ok, Result, anyhow, bail};
use thiserror::Error;

use crate::syntax::ast::Expression;

#[derive(Debug, Error)]
enum TypeInferenceError {
    #[error("Inference impossible: {0}")]
    Impossible(Expression),
    #[error("Invalid type: {0}")]
    InvalidType(Expression),
    #[error("Undefined variable: {0}")]
    UndefinedVariable(Expression),
}

pub fn type_inference(
    type_environment: TypeEnvironment,
    expression: Expression,
) -> Result<(TypeEnvironment, Type)> {
    let (inferred_environment, inferred_type) = infer(type_environment, expression)?;
    let unified_environment = inferred_environment.unify_equations()?;
    println!("{:?}", unified_environment);
    let normalized_type = unified_environment.normalize_type(HashSet::new(), inferred_type)?;

    Ok((unified_environment, normalized_type))
}

fn infer(
    type_environment: TypeEnvironment,
    expression: Expression,
) -> Result<(TypeEnvironment, Type)> {
    match expression {
        Expression::Integer(_) => infer_integer(type_environment, expression),
        Expression::Bool(_) => infer_bool(type_environment, expression),
        Expression::Variable(_) => infer_variable(type_environment, expression),
        Expression::Plus {
            expression1,
            expression2,
        } => infer_binary_operation(type_environment, *expression1, *expression2),
        Expression::Minus {
            expression1,
            expression2,
        } => infer_binary_operation(type_environment, *expression1, *expression2),
        Expression::Times {
            expression1,
            expression2,
        } => infer_binary_operation(type_environment, *expression1, *expression2),
        Expression::LessThan {
            expression1,
            expression2,
        } => infer_binary_predicate(type_environment, *expression1, *expression2),
        Expression::If {
            predicate,
            consequent,
            alternative,
        } => infer_if(type_environment, *predicate, *consequent, *alternative),
        Expression::Let {
            variable,
            bound,
            body,
        } => infer_let(type_environment, variable, *bound, *body),
        Expression::Fun { parameter, body } => infer_fun(type_environment, parameter, *body),
        Expression::App { function, argument } => infer_app(type_environment, *function, *argument),
        Expression::LetRec {
            variable,
            bound_function,
            body,
        } => todo!(),
        Expression::Nil => todo!(),
        Expression::Cons { car, cdr } => todo!(),
        Expression::Match {
            scrutinee,
            nil_case,
            cons_pattern,
        } => todo!(),
    }
}

fn infer_integer(
    type_environment: TypeEnvironment,
    expression: Expression,
) -> Result<(TypeEnvironment, Type)> {
    match expression {
        Expression::Integer(_) => Ok((type_environment, Type::Base(BaseType::Integer))),
        _ => bail!(TypeInferenceError::Impossible(expression)),
    }
}

fn infer_bool(
    type_environment: TypeEnvironment,
    expression: Expression,
) -> Result<(TypeEnvironment, Type)> {
    match expression {
        Expression::Bool(_) => Ok((type_environment, Type::Base(BaseType::Bool))),
        _ => bail!(TypeInferenceError::Impossible(expression)),
    }
}

fn infer_variable(
    type_environment: TypeEnvironment,
    expression: Expression,
) -> Result<(TypeEnvironment, Type)> {
    match &expression {
        Expression::Variable(name) => {
            let variable_type = type_environment
                .get_variable_type(name)
                .ok_or(anyhow!(TypeInferenceError::UndefinedVariable(expression)))?;

            Ok((type_environment, variable_type))
        }
        _ => bail!(TypeInferenceError::UndefinedVariable(expression)),
    }
}

fn infer_binary_operation(
    type_environment: TypeEnvironment,
    expression1: Expression,
    expression2: Expression,
) -> Result<(TypeEnvironment, Type)> {
    let (type_environment, expression1_type) = infer(type_environment, expression1)?;
    let (type_environment, expression2_type) = infer(type_environment, expression2)?;

    let type_environment =
        type_environment.add_equation(expression1_type.clone(), expression2_type.clone());

    Ok((type_environment, expression1_type))
}

fn infer_binary_predicate(
    type_environment: TypeEnvironment,
    expression1: Expression,
    expression2: Expression,
) -> Result<(TypeEnvironment, Type)> {
    let (type_environment, expression1_type) = infer(type_environment, expression1)?;
    let (type_environment, expression2_type) = infer(type_environment, expression2)?;

    let type_environment =
        type_environment.add_equation(expression1_type.clone(), expression2_type.clone());

    Ok((type_environment, Type::Base(BaseType::Bool)))
}

fn infer_if(
    type_environment: TypeEnvironment,
    predicate: Expression,
    consequent: Expression,
    alternative: Expression,
) -> Result<(TypeEnvironment, Type)> {
    let (type_environment, predicate_type) = infer(type_environment, predicate.clone())?;
    let type_environment =
        type_environment.add_equation(predicate_type.clone(), Type::Base(BaseType::Bool));

    let (type_environment, consequent_type) = infer(type_environment, consequent.clone())?;
    let (type_environment, alternative_type) = infer(type_environment, alternative.clone())?;

    let type_environment =
        type_environment.add_equation(consequent_type.clone(), alternative_type.clone());

    Ok((type_environment, consequent_type))
}

fn infer_let(
    type_environment: TypeEnvironment,
    variable: String,
    bound: Expression,
    body: Expression,
) -> Result<(TypeEnvironment, Type)> {
    let (type_environment, bound_type) = infer(type_environment, bound)?;
    let type_environment = type_environment.substitute_variable(variable, bound_type)?;

    infer(type_environment, body)
}

fn infer_fun(
    type_environment: TypeEnvironment,
    parameter: String,
    body: Expression,
) -> Result<(TypeEnvironment, Type)> {
    let unique_parameter = unique_simbol();

    let parameter_type = Type::Variable {
        name: unique_parameter.clone(),
    };

    let type_environment =
        type_environment.substitute_variable(parameter.clone(), parameter_type.clone())?;

    let (type_environment, body_type) = infer(type_environment, body)?;
    let substitued_body_type = body_type.apply_substitution(parameter, unique_parameter);

    Ok((
        type_environment,
        Type::Function {
            domain: parameter_type.clone().into(),
            range: substitued_body_type.into(),
        },
    ))
}

fn infer_app(
    type_environment: TypeEnvironment,
    function: Expression,
    argument: Expression,
) -> Result<(TypeEnvironment, Type)> {
    let (type_environment, function_type) = infer(type_environment, function.clone())?;
    let Type::Function { domain, range } = function_type else {
        bail!(TypeInferenceError::InvalidType(function));
    };

    let (type_environment, argument_type) = infer(type_environment, argument.clone())?;
    let type_environment = type_environment.add_equation(*domain, argument_type);

    Ok((type_environment, *range))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_infer_integer() {
        let expr = Expression::Integer(10);

        let result = type_inference(TypeEnvironment::default(), expr);

        assert!(result.is_ok());
        let (_, ty) = result.unwrap();
        assert_eq!(ty, Type::Base(BaseType::Integer));
    }

    #[test]
    fn test_infer_bool() {
        let expr = Expression::Bool(true);

        let result = type_inference(TypeEnvironment::default(), expr);

        assert!(result.is_ok());
        let (_, ty) = result.unwrap();
        assert_eq!(ty, Type::Base(BaseType::Bool));
    }

    #[test]
    fn test_infer_variable() {
        let env = TypeEnvironment::default()
            .substitute_variable("x".to_string(), Type::Base(BaseType::Integer))
            .unwrap();

        let expr = Expression::Variable("x".to_string());

        let result = infer(env, expr);

        assert!(result.is_ok());
        let (_, ty) = result.unwrap();
        assert_eq!(ty, Type::Base(BaseType::Integer));
    }

    #[test]
    fn test_infer_undefined_variable() {
        let expr = Expression::Variable("unknown".to_string());

        let result = type_inference(TypeEnvironment::default(), expr);

        assert!(result.is_err());
    }

    #[test]
    fn test_infer_plus() {
        let expr = Expression::Plus {
            expression1: Expression::Integer(3).into(),
            expression2: Expression::Integer(5).into(),
        };

        let result = type_inference(TypeEnvironment::default(), expr);

        assert!(result.is_ok());
        let (_, ty) = result.unwrap();
        assert_eq!(ty, Type::Base(BaseType::Integer));
    }

    #[test]
    fn test_infer_minus() {
        let expr = Expression::Minus {
            expression1: Expression::Integer(10).into(),
            expression2: Expression::Integer(5).into(),
        };

        let result = type_inference(TypeEnvironment::default(), expr);

        assert!(result.is_ok());
        let (_, ty) = result.unwrap();
        assert_eq!(ty, Type::Base(BaseType::Integer));
    }

    #[test]
    fn test_infer_times() {
        let expr = Expression::Times {
            expression1: Expression::Integer(3).into(),
            expression2: Expression::Integer(5).into(),
        };

        let result = type_inference(TypeEnvironment::default(), expr);

        assert!(result.is_ok());
        let (_, ty) = result.unwrap();
        assert_eq!(ty, Type::Base(BaseType::Integer));
    }

    #[test]
    fn test_infer_less_than() {
        let expr = Expression::LessThan {
            expression1: Expression::Integer(3).into(),
            expression2: Expression::Integer(5).into(),
        };

        let result = type_inference(TypeEnvironment::default(), expr);

        assert!(result.is_ok());
        let (_, ty) = result.unwrap();
        assert_eq!(ty, Type::Base(BaseType::Bool));
    }

    #[test]
    fn test_infer_invalid_operation() {
        let expr = Expression::Plus {
            expression1: Expression::Integer(3).into(),
            expression2: Expression::Bool(true).into(),
        };

        let result = type_inference(TypeEnvironment::default(), expr);

        assert!(result.is_err());
    }

    #[test]
    fn test_infer_if() {
        let expr = Expression::If {
            predicate: Expression::Bool(true).into(),
            consequent: Expression::Integer(1).into(),
            alternative: Expression::Integer(2).into(),
        };

        let result = type_inference(TypeEnvironment::default(), expr);

        assert!(result.is_ok());
        let (_, ty) = result.unwrap();
        assert_eq!(ty, Type::Base(BaseType::Integer));
    }

    #[test]
    fn test_infer_if_with_invalid_predicate() {
        let expr = Expression::If {
            predicate: Expression::Integer(1).into(),
            consequent: Expression::Integer(1).into(),
            alternative: Expression::Integer(2).into(),
        };

        let result = type_inference(TypeEnvironment::default(), expr);

        assert!(result.is_err());
    }

    #[test]
    fn test_infer_if_with_mismatched_branches() {
        let expr = Expression::If {
            predicate: Expression::Bool(true).into(),
            consequent: Expression::Integer(1).into(),
            alternative: Expression::Bool(false).into(),
        };

        let result = type_inference(TypeEnvironment::default(), expr);

        assert!(result.is_err());
    }

    #[test]
    fn test_infer_let() {
        let expr = Expression::Let {
            variable: "x".to_string(),
            bound: Expression::Integer(5).into(),
            body: Expression::Plus {
                expression1: Expression::Variable("x".to_string()).into(),
                expression2: Expression::Integer(3).into(),
            }
            .into(),
        };

        let result = type_inference(TypeEnvironment::default(), expr);

        assert!(result.is_ok());
        let (_, ty) = result.unwrap();
        assert_eq!(ty, Type::Base(BaseType::Integer));
    }

    #[test]
    fn test_infer_let_with_complex_bound() {
        let expr = Expression::Let {
            variable: "result".to_string(),
            bound: Expression::If {
                predicate: Expression::Bool(true).into(),
                consequent: Expression::Integer(10).into(),
                alternative: Expression::Integer(20).into(),
            }
            .into(),
            body: Expression::Variable("result".to_string()).into(),
        };

        let result = type_inference(TypeEnvironment::default(), expr);

        assert!(result.is_ok());
        let (_, ty) = result.unwrap();
        assert_eq!(ty, Type::Base(BaseType::Integer));
    }

    #[test]
    fn test_infer_fun() {
        let expr = Expression::Fun {
            parameter: "x".to_string(),
            body: Expression::Plus {
                expression1: Expression::Variable("x".to_string()).into(),
                expression2: Expression::Integer(1).into(),
            }
            .into(),
        };

        let result = type_inference(TypeEnvironment::default(), expr);

        assert!(result.is_ok());
        let (_, ty) = result.unwrap();

        match ty {
            Type::Function { domain, range } => {
                assert_eq!(*domain, Type::Base(BaseType::Integer));
                assert_eq!(*range, Type::Base(BaseType::Integer));
            }
            _ => panic!("Expected function type, got: {:?}", ty),
        }
    }

    #[test]
    fn test_infer_identity_fun() {
        let expr = Expression::Fun {
            parameter: "x".to_string(),
            body: Expression::Variable("x".to_string()).into(),
        };

        let result = type_inference(TypeEnvironment::default(), expr);

        assert!(result.is_ok());
        let (_, ty) = result.unwrap();

        match ty {
            Type::Function { domain, range } => {
                assert_eq!(*domain, *range);
            }
            _ => panic!("Expected function type, got: {:?}", ty),
        }
    }

    #[test]
    fn test_infer_app() {
        let function_expr = Expression::Fun {
            parameter: "x".to_string(),
            body: Expression::Plus {
                expression1: Expression::Variable("x".to_string()).into(),
                expression2: Expression::Integer(1).into(),
            }
            .into(),
        };

        let arg_expr = Expression::Integer(5);

        let expr = Expression::App {
            function: function_expr.into(),
            argument: arg_expr.into(),
        };

        let result = type_inference(TypeEnvironment::default(), expr);

        assert!(result.is_ok());
        let (_, ty) = result.unwrap();
        assert_eq!(ty, Type::Base(BaseType::Integer));
    }

    #[test]
    fn test_infer_app_with_invalid_argument() {
        let function_expr = Expression::Fun {
            parameter: "x".to_string(),
            body: Expression::Plus {
                expression1: Expression::Variable("x".to_string()).into(),
                expression2: Expression::Integer(1).into(),
            }
            .into(),
        };

        let arg_expr = Expression::Bool(true);

        let expr = Expression::App {
            function: function_expr.into(),
            argument: arg_expr.into(),
        };

        let result = type_inference(TypeEnvironment::default(), expr);

        assert!(result.is_err());
    }

    #[test]
    fn test_infer_let_rec() {
        let expr = Expression::LetRec {
            variable: "fact".to_string(),
            bound_function: Expression::Fun {
                parameter: "n".to_string(),
                body: Expression::If {
                    predicate: Expression::LessThan {
                        expression1: Expression::Variable("n".to_string()).into(),
                        expression2: Expression::Integer(1).into(),
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

        let result = type_inference(TypeEnvironment::default(), expr);

        assert!(result.is_ok());
        let (_, ty) = result.unwrap();
        assert_eq!(ty, Type::Base(BaseType::Integer));
    }

    // #[test]
    // fn test_infer_nil() {
    //     let expr = Expression::Nil;

    //     let result = type_inference(TypeEnvironment::default(), expr);

    //     assert!(result.is_ok());
    //     let (_, ty) = result.unwrap();
    //     assert!(matches!(ty, Type::Cons));
    // }

    // #[test]
    // fn test_infer_cons() {
    //     let expr = Expression::Cons {
    //         car: Expression::Integer(1).into(),
    //         cdr: Expression::Nil.into(),
    //     };

    //     let result = type_inference(TypeEnvironment::default(), expr);

    //     assert!(result.is_ok());
    //     let (_, ty) = result.unwrap();
    //     assert!(matches!(ty, Type::Cons));
    // }

    #[test]
    fn test_infer_cons_with_invalid_elements() {
        let expr = Expression::Cons {
            car: Expression::Integer(1).into(),
            cdr: Expression::Cons {
                car: Expression::Bool(true).into(),
                cdr: Expression::Nil.into(),
            }
            .into(),
        };

        let result = type_inference(TypeEnvironment::default(), expr);

        assert!(result.is_err());
    }

    #[test]
    fn test_infer_match_nil_case() {
        let expr = Expression::Match {
            scrutinee: Expression::Nil.into(),
            nil_case: Expression::Integer(0).into(),
            cons_pattern: (
                "head".to_string(),
                "tail".to_string(),
                Expression::Plus {
                    expression1: Expression::Variable("head".to_string()).into(),
                    expression2: Expression::Integer(1).into(),
                }
                .into(),
            ),
        };

        let result = type_inference(TypeEnvironment::default(), expr);

        assert!(result.is_ok());
        let (_, ty) = result.unwrap();
        assert_eq!(ty, Type::Base(BaseType::Integer));
    }

    #[test]
    fn test_infer_match_cons_case() {
        let expr = Expression::Match {
            scrutinee: Expression::Cons {
                car: Expression::Integer(1).into(),
                cdr: Expression::Nil.into(),
            }
            .into(),
            nil_case: Expression::Integer(0).into(),
            cons_pattern: (
                "head".to_string(),
                "tail".to_string(),
                Expression::Plus {
                    expression1: Expression::Variable("head".to_string()).into(),
                    expression2: Expression::Integer(1).into(),
                }
                .into(),
            ),
        };

        let result = type_inference(TypeEnvironment::default(), expr);

        assert!(result.is_ok());
        let (_, ty) = result.unwrap();
        assert_eq!(ty, Type::Base(BaseType::Integer));
    }

    #[test]
    fn test_infer_match_with_invalid_scrutinee() {
        let expr = Expression::Match {
            scrutinee: Expression::Integer(5).into(),
            nil_case: Expression::Integer(0).into(),
            cons_pattern: (
                "head".to_string(),
                "tail".to_string(),
                Expression::Plus {
                    expression1: Expression::Variable("head".to_string()).into(),
                    expression2: Expression::Integer(1).into(),
                }
                .into(),
            ),
        };

        let result = type_inference(TypeEnvironment::default(), expr);

        assert!(result.is_err());
    }

    #[test]
    fn test_infer_match_with_mismatched_cases() {
        let expr = Expression::Match {
            scrutinee: Expression::Nil.into(),
            nil_case: Expression::Integer(0).into(),
            cons_pattern: (
                "head".to_string(),
                "tail".to_string(),
                Expression::Bool(true).into(),
            ),
        };

        let result = type_inference(TypeEnvironment::default(), expr);

        assert!(result.is_err());
    }

    #[test]
    fn test_infer_complex_arithmetic() {
        let expr = Expression::Plus {
            expression1: Expression::Integer(3).into(),
            expression2: Expression::Times {
                expression1: Expression::Integer(5).into(),
                expression2: Expression::Integer(2).into(),
            }
            .into(),
        };

        let result = type_inference(TypeEnvironment::default(), expr);

        assert!(result.is_ok());
        let (_, ty) = result.unwrap();
        assert_eq!(ty, Type::Base(BaseType::Integer));
    }

    #[test]
    fn test_infer_complex_predicate() {
        let expr = Expression::LessThan {
            expression1: Expression::Plus {
                expression1: Expression::Integer(3).into(),
                expression2: Expression::Integer(5).into(),
            }
            .into(),
            expression2: Expression::Integer(10).into(),
        };

        let result = type_inference(TypeEnvironment::default(), expr);

        assert!(result.is_ok());
        let (_, ty) = result.unwrap();
        assert_eq!(ty, Type::Base(BaseType::Bool));
    }

    #[test]
    fn test_infer_complex_function() {
        let expr = Expression::Let {
            variable: "add".to_string(),
            bound: Expression::Fun {
                parameter: "x".to_string(),
                body: Expression::Fun {
                    parameter: "y".to_string(),
                    body: Expression::Plus {
                        expression1: Expression::Variable("x".to_string()).into(),
                        expression2: Expression::Variable("y".to_string()).into(),
                    }
                    .into(),
                }
                .into(),
            }
            .into(),
            body: Expression::App {
                function: Expression::App {
                    function: Expression::Variable("add".to_string()).into(),
                    argument: Expression::Integer(3).into(),
                }
                .into(),
                argument: Expression::Integer(4).into(),
            }
            .into(),
        };

        let result = type_inference(TypeEnvironment::default(), expr);

        assert!(result.is_ok());
        let (_, ty) = result.unwrap();
        assert_eq!(ty, Type::Base(BaseType::Integer));
    }

    #[test]
    fn test_infer_list_sum() {
        let sum_function = Expression::Fun {
            parameter: "l".to_string(),
            body: Expression::Match {
                scrutinee: Expression::Variable("l".to_string()).into(),
                nil_case: Expression::Integer(0).into(),
                cons_pattern: (
                    "h".to_string(),
                    "t".to_string(),
                    Expression::Plus {
                        expression1: Expression::Variable("h".to_string()).into(),
                        expression2: Expression::App {
                            function: Expression::Variable("sum".to_string()).into(),
                            argument: Expression::Variable("t".to_string()).into(),
                        }
                        .into(),
                    }
                    .into(),
                ),
            }
            .into(),
        };

        let list_expr = Expression::Cons {
            car: Expression::Integer(1).into(),
            cdr: Expression::Cons {
                car: Expression::Integer(2).into(),
                cdr: Expression::Cons {
                    car: Expression::Integer(3).into(),
                    cdr: Expression::Nil.into(),
                }
                .into(),
            }
            .into(),
        };

        let expr = Expression::LetRec {
            variable: "sum".to_string(),
            bound_function: sum_function.into(),
            body: Expression::App {
                function: Expression::Variable("sum".to_string()).into(),
                argument: list_expr.into(),
            }
            .into(),
        };

        let result = type_inference(TypeEnvironment::default(), expr);

        assert!(result.is_ok());
        let (_, ty) = result.unwrap();
        assert_eq!(ty, Type::Base(BaseType::Integer));
    }
}
