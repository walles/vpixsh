#![allow(clippy::needless_return)]

use std::env;
use std::fs;
use std::io;
use std::io::Write;
use std::os::unix::prelude::ExitStatusExt;
use std::path::PathBuf;
use std::process::Command;

use rustyline::error::ReadlineError;

use crate::ansicolor::{green, red};
use crate::parser::{parse, Executor};

mod ansicolor;
mod cd;
mod parser;
mod tokenizer;

struct Shell {
    current_dir: PathBuf,
    oldpwd: PathBuf,

    /// Ref: https://crates.io/crates/rustyline/#user-content-example
    readline: rustyline::Editor<()>,

    /// Contains an exit status in string form, or a signal name, or the empty
    /// string if the last command succeeded.
    ///
    /// Will be displayed as part of the prompt.
    last_command_exit_description: String,
}

/// Loads history if we have any
fn create_readline() -> rustyline::Editor<()> {
    let mut readline = rustyline::Editor::<()>::new();

    let maybe_homedir = env::var_os("HOME");
    if maybe_homedir == None {
        println!("WARNING: Home directory not set, cannot load history");
        return readline;
    }

    let mut history_path = PathBuf::from(maybe_homedir.unwrap());
    history_path.push(".vpixsh/history");
    if history_path.exists() {
        if let Err(error) = readline.load_history(&history_path) {
            println!(
                "WARNING: Failed to read history from {:?}: {}",
                history_path, error
            );
        }
    }

    return readline;
}

/// Save history
fn drop_readline(readline: &mut rustyline::Editor<()>) {
    // FIXME: Take this value from some internal HOME variable? So if the user
    // sets HOME to some value while inside of the shell that's the HOME we'll
    // use for saving?
    let maybe_homedir = env::var_os("HOME");
    if maybe_homedir == None {
        println!("WARNING: Home directory not set, cannot save history");
        return;
    }

    let mut history_dir = PathBuf::from(maybe_homedir.unwrap());
    history_dir.push(".vpixsh");
    if let Err(error) = fs::create_dir_all(&history_dir) {
        println!(
            "WARNING: Cannot create settings directory {:?}, cannot save history: {}",
            history_dir, error
        );
        return;
    }

    let mut history_path = history_dir.clone();
    history_path.push("history");

    if let Err(error) = readline.save_history(&history_path) {
        println!(
            "WARNING: Saving history into {:?} failed: {}",
            history_path, error
        );
        return;
    }
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
            readline: create_readline(),
            last_command_exit_description: "".to_string(),
        };
    }

    fn run(&mut self) {
        loop {
            // FIXME: Print a colorful prompt with VCS info when available
            println!();
            println!("{}", green(&self.current_dir.to_string_lossy()));

            let mut error_prefix = "".to_string();
            if !self.last_command_exit_description.is_empty() {
                error_prefix = format!(
                    "{} ",
                    red(&format!("[{}]", &self.last_command_exit_description))
                );
            }

            // FIXME: The "$" should be a "#" if we're root
            let prompt = format!("{}$ ", error_prefix);

            // Flush our prompt so the user can see it, necessary since the prompt
            // doesn't end with a newline
            io::stdout().flush().unwrap();

            // Read a line from stdin
            match self.readline.readline(&prompt) {
                Ok(line) => {
                    self.readline.add_history_entry(&line);
                    if let Err(error) = parse(&line, self) {
                        println!("Parse error: {}", error);
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    // FIXME: How should we handle this?
                    println!("CTRL-C");
                    break;
                }
                Err(ReadlineError::Eof) => {
                    // EOF, seeya!
                    println!();
                    break;
                }
                Err(err) => {
                    // If this fails the shell cannot function any more
                    panic!("Failed to read line from stdin: {}", err);
                }
            }
        }
    }

    /// Returns an error message for the prompt, or "" on success
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
            if let Some(os_error) = error.raw_os_error() {
                if os_error == 2 {
                    // "2" == ENOENT
                    return "Not found".to_string();
                }
            }
            return error.to_string();
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
        } else if let Some(signal) = exit_status.signal() {
            // FIXME: Present pretty signal names when available
            return format!("SIG{}", signal);
        } else if let Some(exitcode) = exit_status.code() {
            return format!("{}", exitcode);
        } else {
            return format!("{}", exit_status);
        }
    }
}

impl Drop for Shell {
    fn drop(&mut self) {
        drop_readline(&mut self.readline);
    }
}

impl Executor for Shell {
    fn execute(&mut self, executable: &str, args: &[String]) {
        self.last_command_exit_description = self.do_execute(executable, args);
    }
}

fn main() {
    let mut shell = Shell::new();
    shell.run();
}
