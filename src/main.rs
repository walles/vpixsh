#[macro_use]
extern crate lalrpop_util;

lalrpop_mod!(pub commandline); // synthesized by LALRPOP

mod parser;

fn main() {
    commandline::TermParser::new().parse("22").is_ok();
    println!("Hello World!");
}
