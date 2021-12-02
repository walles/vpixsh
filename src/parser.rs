struct StringAtIndex {
    byte_index: usize,
    string: String,
}

impl StringAtIndex {
    fn after_last_byte_index(&self) -> usize {
        return self.byte_index + self.string.len();
    }
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

fn parse(commandline: &str, executor: &mut dyn Executor) {
    let split: Vec<_> = commandline
        .split_whitespace()
        .map(move |sub| StringAtIndex {
            byte_index: addr_of(sub) - addr_of(commandline),
            string: sub.to_string(),
        })
        .collect();

    if split.is_empty() {
        return;
    }

    executor.execute(&split[0], &split[1..]);
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

    fn parse_into_testrep(commandline: &str) -> Vec<String> {
        let mut test_executor: TestExecutor = TestExecutor::new();
        parse(commandline, &mut test_executor);

        return test_executor.executions;
    }

    #[test]
    fn test_parse_base() {
        assert_eq!(parse_into_testrep("echo"), ["exec('echo')"]);
        assert_eq!(parse_into_testrep("echo hej"), ["exec('echo', 'hej')"]);
        assert_eq!(
            parse_into_testrep("echo hej nej"),
            ["exec('echo', 'hej', 'nej')"]
        );
    }
}
