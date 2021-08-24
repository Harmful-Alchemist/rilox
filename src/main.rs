mod expr;
mod interpreter;
mod lox;
mod loxvalue;
mod parser;
mod scanner;
mod token;
mod tokentype;

use crate::expr::Expr;
use crate::expr::{Binary, Grouping, Literal, Unary};
use crate::lox::Lox;
use crate::loxvalue::LoxValue;
use crate::token::Token;
use crate::tokentype::TokenType;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut lox: Lox = Lox::new();

    // pretty print testing
    // let expression = Binary {
    //     left: &Unary {
    //         operator: Token {
    //             token_type: TokenType::Minus,
    //             lexeme: "-".to_string(),
    //             literal: Literal::None,
    //             line: 1,
    //         },
    //         right: &LiteralExpr {
    //             value: Literal::Number(123 as f64)
    //         },
    //     },
    //     operator: Token {
    //         token_type: TokenType::Star,
    //         lexeme: "*".to_string(),
    //         literal: Literal::None,
    //         line: 1,
    //     },
    //     right: &Grouping { expression: &LiteralExpr { value: Literal::Number(45.67) } },
    // };
    //
    // println!("{}", expression.pretty_print());

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
