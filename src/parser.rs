use nom::Slice;
use nom_locate::LocatedSpan;

enum TokenKind {
    Word,
}
struct Token<'a> {
    text: LocatedSpan<&'a str, ()>,
    kind: TokenKind,
}

pub(crate) trait Executor {
    /// command is the binary to executs
    ///
    /// argv is all the command line arguments. argv does *not* include the
    /// command itself, and will be empty if no arguments are required.
    fn execute(&mut self, command: &str, args: &[String]);
}

/// Implementation of these ten steps:
/// https://pubs.opengroup.org/onlinepubs/9699919799/utilities/V3_chap02.html#tag_18_03
fn string_to_tokens<'a>(input: &'a LocatedSpan<&'a str, ()>) -> Vec<Token<'a>> {
    let mut result: Vec<Token> = vec![];
    let mut token_start: usize = 0; // Byte index

    for (byteindex, character) in input.char_indices() {
        // Rule 7
        if character == ' ' {
            if token_start < byteindex {
                // We were inside a token
                result.push(Token {
                    text: input.slice(token_start..byteindex),
                    kind: TokenKind::Word,
                });
            }
            token_start = byteindex + character.len_utf8();
            continue;
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
            text: input.slice(token_start..),

            // FIXME: How can we know this is a word? Add tests!
            kind: TokenKind::Word,
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
pub(crate) fn parse(commandline: &str, executor: &mut dyn Executor) -> String {
    let spanned_commandline = LocatedSpan::new(commandline);
    let tokens = string_to_tokens(&spanned_commandline);
    if tokens.is_empty() {
        return " ".repeat(commandline.len());
    }

    let mut words: Vec<String> = Vec::new();
    for token in &tokens {
        match token.kind {
            TokenKind::Word => words.push(token.text.to_string()),
        }
    }
    executor.execute(&words[0], &words[1..]);

    let mut highlights = vec![b' '; commandline.chars().count()];
    for (index, token) in tokens.iter().enumerate() {
        let highlighting_code: u8;
        if index == 0 {
            highlighting_code = b'0';
        } else if index % 2 == 1 {
            highlighting_code = b'a';
        } else {
            highlighting_code = b'A';
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

    #[test]
    fn test_parse_utf8() {
        assert_eq!(
            parse_into_testrep("ödla hår är"),
            (
                vec!["exec('ödla', 'hår', 'är')".to_string()],
                "0000 aaa AA".to_string()
            )
        );
    }

    #[test]
    fn test_parse_extra_spacing() {
        assert_eq!(
            parse_into_testrep(" echo  apa   "),
            (
                vec!["exec('echo', 'apa')".to_string()],
                " 0000  aaa   ".to_string()
            )
        );
    }
}
