use crate::tokenizer::{to_tokens, TokenizerError};

pub(crate) trait Executor {
    /// command is the binary to executs
    ///
    /// argv is all the command line arguments. argv does *not* include the
    /// command itself, and will be empty if no arguments are required.
    fn execute(&mut self, command: &str, args: &[String]);
}

/// Returns a string of the same length as the command line, containing
/// highlighting information.
///
/// # Highlighting codes
/// * `0` Executable command
/// * `a` First argument, third, fifth etc...
/// * `A` Second argument, fourth, sixth etc...
/// * `c` Comment
/// * `x` Operator
pub(crate) fn parse<'a>(
    commandline: &'a str,
    executor: &'a mut dyn Executor,
) -> Result<String, TokenizerError<'a>> {
    let tokens_result = to_tokens(commandline);
    if let Err(error) = tokens_result {
        return Err(error);
    }

    let tokens = tokens_result.unwrap();
    if tokens.is_empty() {
        return Ok(" ".repeat(commandline.len()));
    }

    let mut words: Vec<String> = Vec::new();
    for token in &tokens {
        if !token.is_comment {
            words.push(token.text.to_string())
        }
    }
    executor.execute(&words[0], &words[1..]);

    let mut highlights = vec![b' '; commandline.chars().count()];
    let mut word_index: usize = 0;
    for token in tokens.iter() {
        let highlighting_code: u8;
        if token.is_comment {
            highlighting_code = b'c';
        } else {
            // This is a word, not a comment
            if word_index == 0 {
                highlighting_code = b'0';
            } else if word_index % 2 == 1 {
                highlighting_code = b'a';
            } else {
                highlighting_code = b'A';
            }
            word_index += 1;
        }

        let first_char_index = token.text.get_utf8_column() - 1;
        let last_char_index = token.text.get_utf8_column() + token.text.chars().count() - 2;
        #[allow(clippy::needless_range_loop)]
        for i in first_char_index..(last_char_index + 1) {
            highlights[i] = highlighting_code;
        }
    }

    return Ok(String::from_utf8(highlights).unwrap());
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    struct TestExecutor {
        executions: Vec<String>,
    }

    impl Executor for TestExecutor {
        fn execute(&mut self, command: &str, args: &[String]) {
            let mut command_with_args: Vec<String> = vec![command.to_owned()];

            for arg in args {
                command_with_args.push(arg.to_owned());
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
    fn record_execs(commandline: &str) -> (Vec<String>, String) {
        let mut test_executor: TestExecutor = TestExecutor::new();
        let highlights = parse(commandline, &mut test_executor).unwrap();

        return (test_executor.executions, highlights);
    }

    #[test]
    fn test_parse_base() {
        assert_eq!(
            record_execs("echo"),
            (vec!["exec('echo')".to_string()], "0000".to_string())
        );

        assert_eq!(
            record_execs("echo hej"),
            (
                vec!["exec('echo', 'hej')".to_string()],
                "0000 aaa".to_string()
            )
        );

        assert_eq!(
            record_execs("echo hej nej"),
            (
                vec!["exec('echo', 'hej', 'nej')".to_string()],
                "0000 aaa AAA".to_string()
            )
        );
    }

    #[test]
    fn test_parse_utf8() {
        assert_eq!(
            record_execs("??dla h??r ??r"),
            (
                vec!["exec('??dla', 'h??r', '??r')".to_string()],
                "0000 aaa AA".to_string()
            )
        );
    }

    #[test]
    fn test_parse_extra_spacing() {
        assert_eq!(
            record_execs(" echo  apa   "),
            (
                vec!["exec('echo', 'apa')".to_string()],
                " 0000  aaa   ".to_string()
            )
        );
    }

    #[test]
    fn test_comment() {
        assert_eq!(
            record_execs("echo apa#"),
            (
                vec!["exec('echo', 'apa')".to_string()],
                "0000 aaac".to_string()
            )
        );

        assert_eq!(
            record_execs("echo apa #grisar"),
            (
                vec!["exec('echo', 'apa')".to_string()],
                "0000 aaa ccccccc".to_string()
            )
        );
    }
}
