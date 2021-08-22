mod lox;
mod token;
mod tokentype;
mod scanner;
mod literal;

use crate::lox::Lox;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut lox: Lox = Lox { had_error: false };
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

