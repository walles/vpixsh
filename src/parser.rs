use nom::Slice;
use nom_locate::LocatedSpan;

struct Token<'a> {
    text: LocatedSpan<&'a str, ()>,
    is_comment: bool,
}

pub(crate) trait Executor {
    /// command is the binary to executs
    ///
    /// argv is all the command line arguments. argv does *not* include the
    /// command itself, and will be empty if no arguments are required.
    fn execute(&mut self, command: &str, args: &[String]);
}

/// [Operators can be][1] either [control operators][2] or [redirection
/// operators][3].
///
/// [1]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/V1_chap03.html#tag_03_260
/// [2]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/V1_chap03.html#tag_03_113
/// [3]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/V1_chap03.html#tag_03_318
static OPERATORS: [&str; 18] = [
    "&", "&&", "(", ")", ";", ";;", "\n", "|", "||", // <- Control operators
    "<", ">", ">|", "<<", ">>", "<&", ">&", "<<-", "<>", // <- Redirection operators
];

fn is_start_of_operator(character: char) -> bool {
    for operator in OPERATORS {
        if operator.starts_with(character) {
            return true;
        }
    }

    return false;
}

fn is_operator(candidate: &str) -> bool {
    return OPERATORS.contains(&candidate);
}

/// Implementation of these ten steps:
/// https://pubs.opengroup.org/onlinepubs/9699919799/utilities/V3_chap02.html#tag_18_03
fn to_tokens(input: &str) -> Vec<Token> {
    let spanned_input = LocatedSpan::new(input);
    let mut result: Vec<Token> = vec![];
    let mut token_start: usize = 0; // Byte index

    for (byteindex, character) in input.char_indices() {
        if token_start < byteindex {
            let after_current_token = byteindex + character.len_utf8();
            let token_with_current = &spanned_input[token_start..after_current_token];

            if is_operator(token_with_current) {
                // Rule 2, we're building an operator, keep going
                continue;
            }

            let token_without_current = &spanned_input[token_start..byteindex];
            if is_operator(token_without_current) {
                // Rule 3, we found the end of some operator
                result.push(Token {
                    text: spanned_input.slice(token_start..byteindex),
                    is_comment: false,
                });
                token_start = byteindex;

                // Now we just fall through and keep tokenizing the current character
            }
        }

        // Rule 6
        if is_start_of_operator(character) {
            if token_start < byteindex {
                // We were inside a token, delimit that
                result.push(Token {
                    text: spanned_input.slice(token_start..byteindex),
                    is_comment: false,
                });
            }

            token_start = byteindex;
        }

        // Rule 7
        if character == ' ' {
            if token_start < byteindex {
                // We were inside a token
                result.push(Token {
                    text: spanned_input.slice(token_start..byteindex),
                    is_comment: false,
                });
            }

            // Rule 10, try starting a new token at the next character
            token_start = byteindex + character.len_utf8();
            continue;
        }

        // Rule 9
        if character == '#' {
            if token_start < byteindex {
                // We were in the middle of something, push it!
                result.push(Token {
                    text: spanned_input.slice(token_start..byteindex),
                    is_comment: false,
                });
            }

            result.push(Token {
                text: spanned_input.slice(byteindex..),
                is_comment: true,
            });

            return result;
        }

        // Rule 8
        if token_start <= byteindex {
            // We're inside a word, just keep iterating over that word
            continue;
        }
    }

    if token_start < input.len() {
        // Rule 1
        result.push(Token {
            text: spanned_input.slice(token_start..),
            is_comment: false,
        });
    }

    return result;
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
pub(crate) fn parse(commandline: &str, executor: &mut dyn Executor) -> String {
    let tokens = to_tokens(commandline);
    if tokens.is_empty() {
        return " ".repeat(commandline.len());
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
        let highlights = parse(commandline, &mut test_executor);

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
            record_execs("ödla hår är"),
            (
                vec!["exec('ödla', 'hår', 'är')".to_string()],
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

    fn to_token_strings(commandline: &str) -> Vec<String> {
        return to_tokens(commandline)
            .into_iter()
            .map(|token| token.text.to_string())
            .collect();
    }

    #[test]
    fn test_operator() {
        assert_eq!(
            // There is an <<- operator
            to_token_strings("echo<<--"),
            vec!["echo", "<<-", "-"]
        );

        assert_eq!(
            // There is an <<- operator
            to_token_strings("echo > foo"),
            vec!["echo", ">", "foo"]
        );
    }
}
