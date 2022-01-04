use std::path::Path;

use rustyline::completion::Completer;

use crate::{tokenizer::to_tokens, Shell};

fn do_complete(current_dir: &Path, base: &str) -> Vec<String> {
    return Vec::with_capacity(0);
}

fn find_completion_base(line: &str, pos: usize) -> Option<(&str, usize)> {
    let tokenization_result = to_tokens(line);
    if tokenization_result.is_err() {
        return None;
    }

    let tokens = tokenization_result.unwrap();
    for token in tokens {
        if token.text.location_offset() > pos {
            // Token starts after current position, keep looking
            continue;
        }

        if token.text.location_offset() + token.text.len() < pos {
            // Token ends before current position, keep looking
            continue;
        }

        return Some((token.text.fragment(), token.text.location_offset()));
    }

    return None;
}

impl Completer for Shell {
    type Candidate = String;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        if let Some((base, base_start_index)) = find_completion_base(line, pos) {
            // Base found, try completing
            return Ok((base_start_index, do_complete(&self.current_dir, base)));
        }

        // No base -> no completion
        return Ok((0, Vec::with_capacity(0)));
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::Builder;

    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_find_completion_base() {
        assert_eq!(find_completion_base("cd sr", 5), Some(("sr", 3)));
        assert_eq!(find_completion_base("cd sr", 4), Some(("sr", 3)));
        assert_eq!(find_completion_base("cd sr", 3), Some(("sr", 3)));
        assert_eq!(find_completion_base("cd sr", 2), Some(("cd", 0)));
        assert_eq!(find_completion_base("cd sr", 1), Some(("cd", 0)));
        assert_eq!(find_completion_base("cd sr", 0), Some(("cd", 0)));

        assert_eq!(find_completion_base("cd  sr", 3), None);

        assert_eq!(find_completion_base("", 0), None);
        assert_eq!(find_completion_base("  ", 2), None);
        assert_eq!(find_completion_base("  ", 1), None);
        assert_eq!(find_completion_base("  ", 0), None);
    }

    // FIXME: Test finding the completion base in a string with non-ASCII
    // Unicode characters

    // FIXME: Test finding the completion base in a string containing escaping,
    // by backslash, single quotes or double quotes

    #[test]
    fn test_do_complete() {
        let tmp_dir = Builder::new().prefix("example").tempdir().unwrap();
        fs::create_dir(tmp_dir.path().join("src")).unwrap();
        fs::create_dir(tmp_dir.path().join("src/apa")).unwrap();
        fs::create_dir(tmp_dir.path().join("src/bepa")).unwrap();

        assert_eq!(do_complete(tmp_dir.path(), "sr"), vec!["src/"]);
    }

    // FIXME: Test completing a string containing non-ASCII Unicode characters
}
