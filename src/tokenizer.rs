use nom::Slice;
use nom_locate::LocatedSpan;

pub(crate) struct Token<'a> {
    pub text: LocatedSpan<&'a str, ()>,
    pub is_comment: bool,
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
    result: Vec<Token<'a>>,
    token_start: usize, // Byte index
    byteindex: usize,
}

impl<'a> Tokenizer<'a> {
    fn new(input: &'a str) -> Self {
        return Tokenizer {
            input: LocatedSpan::new(input),
            result: vec![],
            token_start: 0,
            byteindex: 0,
        };
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

    /// Fills in Tokenizer.result by following [these ten steps][1].
    ///
    /// [1]: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/V3_chap02.html#tag_18_03
    fn tokenize(&mut self) {
        for (bi, character) in self.input.char_indices() {
            // FIXME: Is there a better way to keep self.byteindex up to date?
            self.byteindex = bi;

            if self.token_start < self.byteindex {
                // We are building some token

                let after_current_token = self.byteindex + character.len_utf8();
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

            // Rule 6
            if is_start_of_operator(character) {
                self.delimit_token();
            }

            // Rule 7
            if character == ' ' {
                self.delimit_token();

                // Rule 10, try starting a new token at the next character
                self.token_start = self.byteindex + character.len_utf8();
                continue;
            }

            // Rule 9
            if character == '#' {
                self.delimit_token();

                self.result.push(Token {
                    text: self.input.slice(self.byteindex..),
                    is_comment: true,
                });

                return;
            }

            // Rule 8, no code needed for this, we're inside a word, just keep
            // iterating over that word.
        }

        if self.token_start < self.input.len() {
            // Rule 1
            self.result.push(Token {
                text: self.input.slice(self.token_start..),
                is_comment: false,
            });
        }
    }
}

/// Implementation of [these ten steps][1]:
///
/// [1]: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/V3_chap02.html#tag_18_03
pub(crate) fn to_tokens(input: &str) -> Vec<Token> {
    let mut tokenizer = Tokenizer::new(input);
    tokenizer.tokenize();
    return tokenizer.result;
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

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

        assert_eq!(to_token_strings("echo > foo"), vec!["echo", ">", "foo"]);
    }
}
