use anyhow::Result;
use picocaml::{
    analysis::{parser::parse, tokenizer::tokenize},
    execution::{environment::Environment, evaluation::eval},
    type_system::{inference::infer, type_environment::TypeEnvironment},
};
use rustyline::{DefaultEditor, error::ReadlineError};

fn main() -> Result<()> {
    let mut global_type_environment = TypeEnvironment::default();
    let mut global_environment = Environment::default();

    let mut rl = DefaultEditor::new()?;

    let mut code = String::new();
    loop {
        match rl.readline(">> ") {
            Ok(line) => {
                code.push_str(line.as_ref());
                code.push('\n');
            }
            Err(ReadlineError::Eof) => {
                rl.add_history_entry(code.as_str())?;

                match parse(tokenize(code.clone())) {
                    Ok(expression) => {
                        let infered = infer(global_type_environment.clone(), expression.clone());
                        if let Err(e) = infered {
                            eprintln!("{}", e);
                            continue;
                        }
                        let (type_environment, ty) = infered.unwrap();
                        global_type_environment = type_environment;
                        println!("Type: {}", ty);

                        let evaluated = eval(global_environment.clone(), expression.clone());
                        if let Err(e) = evaluated {
                            eprintln!("{}", e);
                            continue;
                        }
                        let (environment, value) = evaluated.unwrap();
                        global_environment = environment;
                        println!("Value: {}", value);
                    }
                    Err(e) => {
                        eprintln!("{}", e);
                        continue;
                    }
                }

                code.clear();
            }
            Err(ReadlineError::Interrupted) => {
                println!("Bye ;)");
                break;
            }
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        }
    }

    Ok(())
}
