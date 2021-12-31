pub(crate) fn green(text: &str) -> String {
    return format!("\x1b[32m{}\x1b[39m", text);
}

pub(crate) fn red(text: &str) -> String {
    return format!("\x1b[31m{}\x1b[39m", text);
}
