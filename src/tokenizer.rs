use std::str::CharIndices;

use nom::Slice;
use nom_locate::LocatedSpan;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Token<'a> {
    pub text: LocatedSpan<&'a str, ()>,
    pub is_comment: bool,
}

#[derive(Debug, PartialEq)]
pub(crate) struct TokenizerError<'a> {
    token: Token<'a>,
    message: String,
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

struct Tokenizer<'a> {
    input: LocatedSpan<&'a str>,
    iterator: CharIndices<'a>,

    /// Byte index of where the token we're currently in starts
    token_start: usize,

    /// Byte index where we're currently at, updated in `next()`
    byteindex: usize,

    /// Character we're currently at, updated in `next()`
    character: char,

    result: Vec<Token<'a>>,
}

impl<'a> Tokenizer<'a> {
    fn new(input: &'a str) -> Self {
        return Tokenizer {
            input: LocatedSpan::new(input),
            result: vec![],
            token_start: 0,
            byteindex: 0,
            iterator: input.char_indices(),
            character: '\0',
        };
    }

    fn next(&mut self) -> bool {
        if let Some((byteindex, character)) = self.iterator.next() {
            self.byteindex = byteindex;
            self.character = character;
            return true;
        }

        return false;
    }

    fn delimit_token(&mut self) {
        if self.token_start < self.byteindex {
            // We are building a token, delimit that
            self.result.push(Token {
                text: self.input.slice(self.token_start..self.byteindex),
                is_comment: false,
            });
        }

        self.token_start = self.byteindex;
    }

    /// Ref:
    /// https://pubs.opengroup.org/onlinepubs/9699919799/utilities/V3_chap02.html#tag_18_02_01
    ///
    /// This function returns `Some(Err(TokenizerError{...}))` on failure, `None`
    /// otherwise.
    #[must_use]
    fn tokenize_backslash_escape(&mut self) -> Option<Result<Vec<Token<'a>>, TokenizerError<'a>>> {
        if self.character != '\\' {
            panic!("Must be at a backslash when calling this method");
        }
        if !self.next() {
            return Some(Err(TokenizerError {
                token: Token {
                    text: self.input.slice(self.byteindex..),
                    is_comment: false,
                },
                message:
                    "Backslash can't be last, remove the backslash or add more characters after it"
                        .to_string(),
            }));
        }

        // Doing nothing here means we keep building the current token, so let's do nothing!
        return None;
    }

    /// Fills in Tokenizer.result by following [these ten steps][1].
    ///
    /// [1]: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/V3_chap02.html#tag_18_03
    fn tokenize(mut self) -> Result<Vec<Token<'a>>, TokenizerError<'a>> {
        while self.next() {
            if self.token_start < self.byteindex {
                // We are building some token

                let after_current_token = self.byteindex + self.character.len_utf8();
                let token_with_current = &self.input[self.token_start..after_current_token];

                if is_operator(token_with_current) {
                    // Rule 2, we're building an operator, keep going
                    continue;
                }

                let token_without_current = &self.input[self.token_start..self.byteindex];
                if is_operator(token_without_current) {
                    // Rule 3, we found the end of some operator
                    self.delimit_token();

                    // Now we just fall through and keep tokenizing the current character
                }
            }

            // Rule 4
            if self.character == '\\' {
                if let Some(error) = self.tokenize_backslash_escape() {
                    return error;
                }
                continue;
            }

            // Rule 6
            if is_start_of_operator(self.character) {
                self.delimit_token();
            }

            // Rule 7
            if self.character == ' ' {
                self.delimit_token();

                // Rule 10, try starting a new token at the next character
                self.token_start = self.byteindex + self.character.len_utf8();
                continue;
            }

            // Rule 9
            if self.character == '#' {
                self.delimit_token();

                self.result.push(Token {
                    text: self.input.slice(self.byteindex..),
                    is_comment: true,
                });

                return Ok(self.result);
            }

            // Rule 8, no code needed for this, we're inside a word, just keep
            // iterating over that word.
        }

        // Rule 1
        self.byteindex = self.input.len();
        self.delimit_token();

        return Ok(self.result);
    }
}

/// Implementation of [these ten steps][1]:
///
/// [1]: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/V3_chap02.html#tag_18_03
pub(crate) fn to_tokens(input: &str) -> Result<Vec<Token>, TokenizerError> {
    return Tokenizer::new(input).tokenize();
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    fn to_token_strings(commandline: &str) -> Vec<String> {
        return to_tokens(commandline)
            .unwrap()
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

        assert_eq!(to_token_strings("echo > foo"), vec!["echo", ">", "foo"]);
    }

    fn assert_parse_error(
        commandline: &str,
        failed_part: &str,
        failed_offset: usize,
        message: &str,
    ) {
        let error = to_tokens(commandline).unwrap_err();

        unsafe {
            let expected_span: LocatedSpan<&str, ()> =
                LocatedSpan::new_from_raw_offset(failed_offset, 1, failed_part, ());

            assert_eq!(
                error,
                TokenizerError {
                    token: Token {
                        text: expected_span,
                        is_comment: false,
                    },
                    message: message.to_string(),
                }
            );
        }
    }

    #[test]
    fn test_backslash_escape() {
        // Note that "the result token shall contain exactly the characters that
        // appear in the input", or in other words, the backslashes stay:
        // https://pubs.opengroup.org/onlinepubs/9699919799/utilities/V3_chap02.html#tag_18_03
        assert_eq!(to_token_strings(r"echo\ hej"), vec![r"echo\ hej"]);
        assert_eq!(to_token_strings(r"echo \> hej"), vec!["echo", r"\>", "hej"]);
        assert_eq!(
            to_token_strings(r"echo >\> hej"),
            vec!["echo", ">", r"\>", "hej"]
        );
        assert_eq!(to_token_strings(r"echo hej\'"), vec![r"echo", r"hej\'"]);

        assert_parse_error(
            r"apa\",
            r"\",
            3,
            "Backslash can't be last, remove the backslash or add more characters after it",
        );
    }

    // FIXME: Add backslash-newline continuation marker test(s)
}
