#![allow(clippy::needless_return)]

use crate::parser::parse;
use crate::parser::Executor;

mod parser;

struct SlaskExecutor {}

impl Executor for SlaskExecutor {
    fn execute(
        &mut self,
        _command: &nom_locate::LocatedSpan<&str, ()>,
        _args: &[nom_locate::LocatedSpan<&str, ()>],
    ) {
        // This method left blank until further notice
    }
}

fn main() {
    println!("Hello World!");

    // For now, this call is here to get rid of some unused-code warnings
    parse("echo", &mut SlaskExecutor {});
}
