#![allow(clippy::needless_return)]

use crate::parser::{parse, Executor, Span};

mod parser;

struct SlaskExecutor {}

impl Executor for SlaskExecutor {
    fn execute(&mut self, command: &Span, args: &[Span]) {
        let mut command_with_args = vec![command.to_string()];

        for arg in args {
            command_with_args.push(arg.to_string());
        }

        println!("exec('{}')", command_with_args.join("', '"));
    }
}

fn main() {
    // For now, this call is here to get rid of some unused-code warnings
    parse("echo hello world", &mut SlaskExecutor {});
}
