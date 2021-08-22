use crate::token::Token;
use crate::tokentype::TokenType;
use crate::lox::Lox;

pub struct Scanner<'a> {
    source: String,
    lox: &'a mut Lox,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,

}

impl<'a> Scanner<'a> {
    pub fn new(source: String, lox: &'a mut Lox) -> Self {
        Scanner {
            source,
            lox,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_tokens(&mut self) -> Vec<Token> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }

        self.tokens.push(Token {
            token_type: TokenType::EOF,
            lexeme: "".to_string(),
            literal: None,
            line: self.line as u64,
        });
        self.tokens.to_vec()
    }

    fn scan_token(&mut self) {
        let c = self.advance();
        match c {
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            ',' => self.add_token(TokenType::Comma),
            '.' => self.add_token(TokenType::Dot),
            '-' => self.add_token(TokenType::Minus),
            '+' => self.add_token(TokenType::Plus),
            ';' => self.add_token(TokenType::SemiColon),
            '*' => self.add_token(TokenType::Star),
            '!' => {
                let doubled = self.match_char('=');
                self.add_token(if doubled { TokenType::BangEqual } else { TokenType::Bang });
            }
            '=' => {
                let doubled = self.match_char('=');
                self.add_token(if doubled { TokenType::EqualEqual } else { TokenType::Equal })
            }
            '<' => {
                let doubled = self.match_char('=');
                self.add_token(if doubled { TokenType::LessEqual } else { TokenType::Less })
            }
            '>' => {
                let doubled = self.match_char('=');
                self.add_token(if doubled { TokenType::GreaterEqual } else { TokenType::Greater })
            }
            '/' => {
                let doubled = self.match_char('/');
                if doubled {
                    let mut next = self.peek();
                    while next != '\n' && !self.is_at_end() {
                        self.advance();
                        next = self.peek();
                    }
                } else {
                    self.add_token(TokenType::Slash);
                }
            }
            ' ' | '\r' | '\t' => (),
            '\n' => self.line = self.line + 1,
            '"' => self.string(),
            _ => self.lox.error(self.line as u64, String::from("Unexpected character."))
        }
    }

    fn string(&mut self) {

    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.source.chars().nth(self.current).unwrap() != expected {
            return false;
        }
        self.current = self.current + 1;
        return true;
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }
        self.source.chars().nth(self.current).unwrap()
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn advance(&mut self) -> char {
        let return_char = self.source.chars().nth(self.current).unwrap(); //TODO not so nice but following along
        self.current = self.current + 1;
        return_char
    }

    fn add_token(&mut self, token_type: TokenType) {
        self.add_token_total(token_type, None);
    }

    fn add_token_total(&mut self, token_type: TokenType, literal: Option<bool>) {
        let text = &self.source[self.start..self.current];
        self.tokens.push(
            Token {
                token_type,
                lexeme: String::from(text),
                literal,
                line: self.line as u64,

            }
        )
    }
}