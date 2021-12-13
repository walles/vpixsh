use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, alphanumeric1, space1},
    combinator::recognize,
    multi::{many0, separated_list1},
    sequence::pair,
    IResult,
};
use nom_locate::LocatedSpan;

pub(crate) trait Executor {
    /// command is the binary to executs
    ///
    /// argv is all the command line arguments. argv does *not* include the
    /// command itself, and will be empty if no arguments are required.
    fn execute(&mut self, command: &LocatedSpan<&str, ()>, args: &[LocatedSpan<&str, ()>]);
}

fn word(input: LocatedSpan<&str, ()>) -> IResult<LocatedSpan<&str, ()>, LocatedSpan<&str, ()>> {
    recognize(pair(
        alt((alpha1, tag("_"))),
        many0(alt((alphanumeric1, tag("_")))),
    ))(input)
}

fn words(
    input: LocatedSpan<&str, ()>,
) -> IResult<LocatedSpan<&str, ()>, Vec<LocatedSpan<&str, ()>>> {
    return separated_list1(space1, word)(input);
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
    let command_words = words(spanned_commandline).unwrap().1;

    if command_words.is_empty() {
        return " ".repeat(commandline.len());
    }

    executor.execute(&command_words[0], &command_words[1..]);

    let mut highlights = vec![b' '; commandline.len()];
    for (index, token) in command_words.iter().enumerate() {
        let highlighting_code: u8;
        if index == 0 {
            highlighting_code = b'0';
        } else if index % 2 == 1 {
            highlighting_code = b'a';
        } else {
            highlighting_code = b'A';
        }

        let first_byte_index = token.location_offset();
        let last_byte_index = token.location_offset() + token.chars().count() - 1;
        #[allow(clippy::needless_range_loop)]
        for i in first_byte_index..(last_byte_index + 1) {
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
        fn execute(&mut self, command: &LocatedSpan<&str, ()>, args: &[LocatedSpan<&str, ()>]) {
            let mut command_with_args = vec![command.to_string()];

            for arg in args {
                command_with_args.push(arg.to_string());
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

    // FIXME: Do test_parse_base with UTF-8 chars in it

    // FIXME: Test with extra spacing: " echo  hej"
}
