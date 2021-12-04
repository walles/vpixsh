// Ref: http://lalrpop.github.io/lalrpop/tutorial/001_adding_lalrpop.html

extern crate lalrpop;

fn main() {
    lalrpop::Configuration::new()
        .generate_in_source_tree()
        .process()
        .unwrap();
}
