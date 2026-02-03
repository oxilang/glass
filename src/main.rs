use std::fs::read_to_string;

use lexer::tokenize;
use parser::parse;

mod ast;
mod lexer;
mod parser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = read_to_string("examples/02.glass").expect("");
    let tokens = tokenize(file)?;
    let ast = parse(tokens)?;
    println!("{:#?}", ast);
    Ok(())
}
