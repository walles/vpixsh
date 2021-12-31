#![allow(clippy::needless_return)]

use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs};

use crate::ansicolor::{green, red};
use crate::parser::{parse, Executor};

mod ansicolor;
mod parser;
mod tokenizer;

struct Shell {
    current_dir: PathBuf,
    oldpwd: PathBuf,

    /// Contains an exit status in string form, or a signal name, or the empty
    /// string if the last command succeeded.
    ///
    /// Will be displayed as part of the prompt.
    last_command_exit_description: String,
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

        return Shell {
            current_dir: current_dir.to_owned(),
            oldpwd: current_dir,
            last_command_exit_description: "".to_string(),
        };
    }

    fn run(&mut self) {
        loop {
            // FIXME: Print a colorful prompt with VCS info when available
            println!();
            println!("{}", green(&self.current_dir.to_string_lossy()));

            if !self.last_command_exit_description.is_empty() {
                print!(
                    "{} ",
                    red(&format!("[{}]", &self.last_command_exit_description))
                );
            }

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

    fn cd(&mut self, args: &[String]) -> String {
        if args.is_empty() {
            let env_home = env::var("HOME");
            if let Err(error) = env_home {
                println!("ERROR: Cannot read HOME environment variable: {}", error);
                return "cd: HOME not set".to_string();
            }
            self.cd(&[env_home.unwrap()]);
            return "".to_string();
        }

        if args.len() != 1 {
            println!("ERROR: cd wanted zero or one argument, got {}", args.len());
            return "Too many args".to_string();
        }

        let target = &args[0];

        if target == "-" {
            let temp = self.current_dir.to_owned();
            self.current_dir = self.oldpwd.to_owned();
            self.oldpwd = temp;
            return "".to_string();
        }

        let mut target_path = PathBuf::from(&self.current_dir);

        // Pushing resolves absolute paths
        target_path.push(Path::new(target));

        if !target_path.is_dir() {
            println!("ERROR: Not a directory: {}", target);
            return "Not a dir".to_string();
        }

        let canonicalize_result = target_path.canonicalize();
        if let Err(error) = canonicalize_result {
            println!(
                "ERROR: Unable to canonicalize <{}>: {}",
                target_path.to_string_lossy(),
                error
            );
            return error.to_string();
        }
        target_path = canonicalize_result.unwrap();

        if let Err(error) = fs::read_dir(target_path.to_owned()) {
            println!(
                "ERROR: Target directory <{}> is inaccessible: {}",
                target_path.to_string_lossy(),
                error
            );
            return error.to_string();
        }

        self.oldpwd = self.current_dir.to_owned();
        self.current_dir = target_path;
        return "".to_string();
    }

    fn do_execute(&mut self, executable: &str, args: &[String]) -> String {
        let mut command_with_args = vec![executable.to_string()];

        let mut command = Command::new(executable);
        command.current_dir(self.current_dir.to_owned());

        // Color BSD "ls" output.
        // FIXME: This isn't very generic. Maybe put this in the default config
        // file with an associated comment?
        command.env("CLICOLOR", "1");

        for arg in args {
            command_with_args.push(arg.to_string());
            command.arg(arg);
        }

        if executable == "cd" {
            return self.cd(args);
        }

        println!("About to do: exec('{}')", command_with_args.join("', '"));
        let exec_result = command.spawn();
        if let Err(error) = exec_result {
            println!("exec() failed: {}", error);
            return "Not found".to_string();
        }

        let mut child = exec_result.unwrap();
        let wait_result = child.wait();
        if let Err(error) = wait_result {
            println!("Awaiting child process failed: {}", error);
            return error.to_string();
        }

        let exit_status = wait_result.unwrap();
        if exit_status.success() {
            return "".to_string();
        } else {
            return format!("{}", exit_status);
        }
    }
}

impl Executor for Shell {
    fn execute(&mut self, executable: &str, args: &[String]) {
        self.last_command_exit_description = self.do_execute(executable, args);
        println!("Exit status: {}", self.last_command_exit_description);
    }
}

fn main() {
    let mut shell = Shell::new();
    shell.run();
}
