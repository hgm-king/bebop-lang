use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};

use bebop_lang::lisp::{Compile, Lisp};

fn main() -> Result<()> {
    // `()` can be used when no completer is required
    let mut rl = DefaultEditor::new()?;
    let mut env = bebop_lang::lisp::env::init_env();
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str())?;
                let v = Lisp::from_source(&mut env, &line.as_str());
                println!("{:?}", v);
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }

    Ok(())
}
