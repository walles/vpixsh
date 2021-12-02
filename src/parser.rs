struct StringAtIndex {
    first_byte_index: usize,
    last_byte_index: usize,
    string: String,
}

trait Executor {
    /// command is the binary to executs
    ///
    /// argv is all the command line arguments. argv does *not* include the
    /// command itself, and will be empty if no arguments are required.
    fn execute(&mut self, command: &StringAtIndex, args: &[StringAtIndex]);
}

fn addr_of(s: &str) -> usize {
    s.as_ptr() as usize
}

/// Returns a string of the same length as the command line, containing
/// highlighting information.
///
/// # Highlighting codes
/// * `0` Executable command
/// * `a` First argument, third, fifth etc...
/// * `A` Second argument, fourth, sixth etc...
fn parse(commandline: &str, executor: &mut dyn Executor) -> String {
    let split: Vec<_> = commandline
        .split_whitespace()
        .map(move |arg| StringAtIndex {
            first_byte_index: addr_of(arg) - addr_of(commandline),
            last_byte_index: arg.len() + addr_of(arg) - addr_of(commandline) - 1,
            string: arg.to_string(),
        })
        .collect();

    if split.is_empty() {
        return " ".repeat(commandline.len());
    }

    executor.execute(&split[0], &split[1..]);

    let mut highlights = vec![b' '; commandline.len()];
    for (index, token) in split.iter().enumerate() {
        let highlighting_code: u8;
        if index == 0 {
            highlighting_code = b'0';
        } else if index % 2 == 1 {
            highlighting_code = b'a';
        } else {
            highlighting_code = b'A';
        }

        #[allow(clippy::needless_range_loop)]
        for i in token.first_byte_index..(token.last_byte_index + 1) {
            highlights[i] = highlighting_code;
        }
    }

    return String::from_utf8(highlights).unwrap();
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    struct TestExecutor {
        executions: Vec<String>,
    }

    impl Executor for TestExecutor {
        fn execute(&mut self, command: &StringAtIndex, args: &[StringAtIndex]) {
            let mut command_with_args = vec![command.string.to_owned()];

            for arg in args {
                command_with_args.push(arg.string.to_owned());
            }

            self.executions
                .push(format!("exec('{}')", command_with_args.join("', '")));
        }
    }

    impl TestExecutor {
        fn new() -> TestExecutor {
            return TestExecutor {
                executions: Vec::new(),
            };
        }
    }

    /// Returns a vector of commands to be executed given this command line
    fn parse_into_testrep(commandline: &str) -> (Vec<String>, String) {
        let mut test_executor: TestExecutor = TestExecutor::new();
        let highlights = parse(commandline, &mut test_executor);

        return (test_executor.executions, highlights);
    }

    #[test]
    fn test_parse_base() {
        assert_eq!(
            parse_into_testrep("echo"),
            (vec!["exec('echo')".to_string()], "0000".to_string())
        );

        assert_eq!(
            parse_into_testrep("echo hej"),
            (
                vec!["exec('echo', 'hej')".to_string()],
                "0000 aaa".to_string()
            )
        );

        assert_eq!(
            parse_into_testrep("echo hej nej"),
            (
                vec!["exec('echo', 'hej', 'nej')".to_string()],
                "0000 aaa AAA".to_string()
            )
        );
    }

    // FIXME: Test with extra spacing: " echo  hej"
}
