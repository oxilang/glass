use std::fs::read_to_string;

use lexer::tokenize;

mod lexer;
mod parser;

fn main() {
    let file = read_to_string("examples/02.glass").expect("");
    let res = tokenize(file);
    println!("{res:#?}")
}
