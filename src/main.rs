mod lox;
mod token;
mod tokentype;
mod scanner;
mod literal;
mod expr;
mod parser;

use crate::lox::Lox;
use std::env;
use crate::expr::{Binary, Unary, Grouping, LiteralExpr};
use crate::expr::Expr;
use crate::token::Token;
use crate::tokentype::TokenType;
use crate::literal::Literal;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut lox: Lox = Lox { had_error: false };

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

