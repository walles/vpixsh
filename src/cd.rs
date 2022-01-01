use std::{
    env, fs,
    path::{Path, PathBuf},
};

use crate::Shell;

impl Shell {
    pub(crate) fn cd(&mut self, args: &[String]) -> String {
        if args.is_empty() {
            let env_home = env::var("HOME");
            if let Err(error) = env_home {
                println!("ERROR: Cannot read HOME environment variable: {}", error);
                return "HOME not set".to_string();
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
            if let Some(os_error) = error.raw_os_error() {
                if os_error == 13 {
                    // "13" == EPERM
                    return "Permission denied".to_string();
                }
            }
            return error.to_string();
        }

        self.oldpwd = self.current_dir.to_owned();
        self.current_dir = target_path;
        return "".to_string();
    }
}
