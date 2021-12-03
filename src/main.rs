#![allow(clippy::needless_return)]

#[macro_use]
extern crate lalrpop_util;

lalrpop_mod!(pub commandline); // synthesized by LALRPOP

mod parser;

fn main() {
    if commandline::CommandlineParser::new()
        .parse("echo hej")
        .is_err()
    {
        println!("Oh noes!");
    }
    println!("Hello World!");
}
