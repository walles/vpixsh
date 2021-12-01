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
        highlighted: Vec<u8>,
    }

    impl Executor for TestExecutor {
        fn execute(&mut self, command: &StringAtIndex, args: &[StringAtIndex]) {
            // Set highlights for command
            for i in command.byte_index..command.after_last_byte_index() {
                self.highlighted[i] = b'0';
            }

            // Set highlights for args
            for (argno, arg) in args.iter().enumerate() {
                for i in arg.byte_index..arg.after_last_byte_index() {
                    self.highlighted[i] = b'1' + (argno as u8);
                }
            }
        }
    }

    impl TestExecutor {
        fn new(commandline: &str) -> TestExecutor {
            return TestExecutor {
                highlighted: vec![b' '; commandline.len()],
            };
        }
    }

    fn parse_into_testrep(commandline: &str) -> String {
        let mut test_executor: TestExecutor = TestExecutor::new(commandline);
        parse(commandline, &mut test_executor);
        return std::str::from_utf8(&test_executor.highlighted)
            .unwrap()
            .to_string();
    }

    #[test]
    fn test_parse_base() {
        assert_eq!(parse_into_testrep("echo hej"), "0000 111");
    }
}
