#![allow(clippy::needless_return)]

use std::env;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::parser::{parse, Executor};

mod parser;
mod tokenizer;

struct Shell {
    current_dir: PathBuf,
}

impl Shell {
    fn new() -> Self {
        let mut current_dir = PathBuf::new();
        let current_dir_result = env::current_dir();
        if let Err(error) = current_dir_result {
            // Just leave the current_dir empty
            println!("ERROR: Failed getting current directory: {}", error);
        } else {
            // Ref: https://stackoverflow.com/a/42579588/473672
            current_dir = current_dir_result.unwrap();
        }

        return Shell { current_dir };
    }

    fn run(&mut self) {
        loop {
            // FIXME: Print a colorful prompt with VCS info when available
            println!();
            println!("{}", self.current_dir.to_string_lossy());

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

            if let Err(error) = parse(&line, self) {
                println!("Parse error: {}", error);
            }
        }
    }

    fn cd(&mut self, args: &[String]) {
        // FIXME: Go to home directory on zero args

        if args.len() != 1 {
            println!("cd wanted one argument, got {}", args.len());
            return;
        }

        let target = &args[0];

        // FIXME: Go to previous directory if "target" is "-"

        let mut target_path = PathBuf::from(&self.current_dir);

        // Pushing resolves absolute paths
        target_path.push(Path::new(target));

        if !target_path.is_dir() {
            println!("Not a directory: {}", target);
            return;
        }

        let canonicalize_result = target_path.canonicalize();
        if let Err(error) = canonicalize_result {
            println!(
                "Unable to canonicalize <{}>: {}",
                target_path.to_string_lossy(),
                error
            );
            return;
        }
        target_path = canonicalize_result.unwrap();

        self.current_dir = target_path;
    }
}

impl Executor for Shell {
    fn execute(&mut self, executable: &str, args: &[String]) {
        let mut command_with_args = vec![executable.to_string()];

        let mut command = Command::new(executable);
        command.current_dir(self.current_dir.to_owned());

        for arg in args {
            command_with_args.push(arg.to_string());
            command.arg(arg);
        }

        if executable == "cd" {
            self.cd(args);
            return;
        }

        println!("About to do: exec('{}')", command_with_args.join("', '"));
        // FIXME: Verify that spawn() honors the $PATH
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
    let mut shell = Shell::new();
    shell.run();
}
