#[macro_use]
extern crate lalrpop_util;

lalrpop_mod!(pub commandline); // synthesized by LALRPOP

mod parser;

fn main() {
    commandline::CommandlineParser::new()
        .parse("echo hej")
        .is_ok();
    println!("Hello World!");
}
