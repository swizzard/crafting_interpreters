use std::process::exit;
fn main() {
    match crafting_interpreters::main() {
        Ok(()) => exit(0),
        Err(err @ crafting_interpreters::InterpreterError::Usage) => {
            println!("{:?}", err);
            exit(64)
        }
        Err(err @ crafting_interpreters::InterpreterError::Interpreter { .. }) => {
            println!("{:?}", err);
            exit(65)
        }
        Err(e) => {
            println!("{:?}", e);
            exit(70)
        }
    }
}
