mod expr;
mod interpreter;
mod lox;
mod loxvalue;
mod parser;
mod scanner;
mod stmt;
mod token;
mod tokentype;

use crate::lox::Lox;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut lox: Lox = Lox::new();

    if args.len() > 2 {
        println!("Usage: rilox [script] ");
        std::process::exit(64);
    } else if args.len() == 2 {
        let source: &String = &args[1];
        lox.run_file(source);
    } else {
        lox.run_prompt();
    }
}
