use std::{env, fs, path::PathBuf};

use crate::Shell;

impl Shell {
    fn cd_minus(&mut self) -> String {
        self.current_dir = self.oldpwd.to_owned();
        return "".to_string();
    }

    fn cd_directory(&mut self, mut target: PathBuf) -> String {
        if target.is_relative() {
            let mut absolute_target = PathBuf::from(&self.current_dir);
            absolute_target.push(target);
            target = absolute_target;
        }

        if !target.is_dir() {
            println!("ERROR: Not a directory: {}", target.to_string_lossy());
            return "Not a dir".to_string();
        }

        let canonicalize_result = target.canonicalize();
        if let Err(error) = canonicalize_result {
            println!(
                "ERROR: Unable to canonicalize <{}>: {}",
                target.to_string_lossy(),
                error
            );
            return error.to_string();
        }
        target = canonicalize_result.unwrap();

        if let Err(error) = fs::read_dir(target.to_owned()) {
            println!(
                "ERROR: Target directory <{}> is inaccessible: {}",
                target.to_string_lossy(),
                error
            );
            if let Some(os_error) = error.raw_os_error() {
                if os_error == 13 {
                    // "13" == EPERM
                    return "Permission denied".to_string();
                }
            }
            return error.to_string();
        }

        self.current_dir = target;
        return "".to_string();
    }

    pub(crate) fn cd(&mut self, args: &[String]) -> String {
        if args.is_empty() {
            let env_home = env::var("HOME");
            if let Err(error) = env_home {
                println!("ERROR: Cannot read HOME environment variable: {}", error);
                return "HOME not set".to_string();
            }
            return self.cd(&[env_home.unwrap()]);
        }

        if args.len() != 1 {
            println!("ERROR: cd wanted zero or one argument, got {}", args.len());
            return "Too many args".to_string();
        }

        let dir_before = self.current_dir.clone();

        let target = &args[0];
        let problem: String;
        if target == "-" {
            problem = self.cd_minus();
        } else {
            problem = self.cd_directory(PathBuf::from(target));
        }

        if problem.is_empty() {
            self.oldpwd = dir_before;
        }
        return problem;
    }
}
