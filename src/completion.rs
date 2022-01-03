use std::path::Path;

use rustyline::completion::Completer;

use crate::Shell;

fn do_complete(current_dir: &Path, base: &str) -> Vec<String> {
    return Vec::with_capacity(0);
}

fn find_completion_base(line: &str, pos: usize) -> Option<(&str, usize)> {
    return None;
}

impl Completer for Shell {
    type Candidate = String;

    fn complete(
        &self, // FIXME should be `&mut self`
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

        assert_eq!(find_completion_base("", 0), None);
        assert_eq!(find_completion_base("  ", 2), None);
        assert_eq!(find_completion_base("  ", 1), None);
        assert_eq!(find_completion_base("  ", 0), None);
    }

    // FIXME: Add do_complete() tests
}
