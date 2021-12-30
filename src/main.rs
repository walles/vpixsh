#![allow(clippy::needless_return)]

use std::env;
use std::io::{self, BufRead, Write};
use std::process::Command;

use crate::parser::{parse, Executor};

mod parser;
mod tokenizer;

struct ExecExecutor {}

impl Executor for ExecExecutor {
    fn execute(&mut self, command: &str, args: &[String]) {
        let mut command_with_args = vec![command.to_string()];
        let mut command = Command::new(command);

        for arg in args {
            command_with_args.push(arg.to_string());
            command.arg(arg);
        }

        println!("About to do: exec('{}')", command_with_args.join("', '"));
        let exec_result = command.spawn();
        if let Err(error) = exec_result {
            println!("exec() failed: {}", error);
            return;
        }

        let mut child = exec_result.unwrap();
        let wait_result = child.wait();
        if let Err(error) = wait_result {
            println!("Awaiting child process failed: {}", error);
            return;
        }

        let exit_status = wait_result.unwrap();
        println!("Exit status: {}", exit_status);
    }
}

fn main() {
    let current_dir: String;
    let current_dir_result = env::current_dir();
    if let Err(error) = current_dir_result {
        current_dir = format!("Failed getting current directory: {}", error);
    } else {
        // Ref: https://stackoverflow.com/a/42579588/473672
        current_dir = current_dir_result
            .unwrap()
            .into_os_string()
            .into_string()
            // FIXME: Will fail if path is not UTF-8
            .unwrap();
    }

    loop {
        // FIXME: Print a usable prompt
        println!();
        println!("{}", current_dir);

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

        if let Err(error) = parse(&line, &mut ExecExecutor {}) {
            println!("Parse error: {}", error);
        }
    }
}
