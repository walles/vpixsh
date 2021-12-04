#![allow(clippy::needless_return)]

mod commandline;
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
