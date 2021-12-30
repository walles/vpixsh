#![allow(clippy::needless_return)]

use std::io::{self, BufRead, Write};

use crate::parser::{parse, Executor};

mod parser;
mod tokenizer;

struct PrintExecutor {}

impl Executor for PrintExecutor {
    fn execute(&mut self, command: &str, args: &[String]) {
        let mut command_with_args = vec![command.to_string()];

        for arg in args {
            command_with_args.push(arg.to_string());
        }

        println!("exec('{}')", command_with_args.join("', '"));
    }
}

fn main() {
    loop {
        // FIXME: Print a usable prompt
        println!();
        println!("/current/path/goes/here");

        // FIXME: Should be # if we're root
        print!("$ ");

        // Flush our prompt so the user can see it, necessary since the prompt
        // doesn't end with a newline
        if let Err(error) = io::stdout().flush() {
            panic!("{}", error)
        }

        // Read a line from stdin
        // Ref: https://stackoverflow.com/a/30186553/473672
        let maybe_line = io::stdin().lock().lines().next();
        if maybe_line.is_none() {
            // EOF, seeya!
            println!();
            break;
        }

        // FIXME: Second unwrap is of a Result<String, Error>. I don't know what
        // that error could be, let's fix that when it happens!
        let line = maybe_line.unwrap().unwrap();

        if let Err(error) = parse(&line, &mut PrintExecutor {}) {
            println!("Parse error: {}", error);
        }
    }
}
