use crate::expr::{Assign, Binary, Expr, Grouping, Kind, Literal, NoOp, Unary, Variable};
use crate::loxvalue::LoxValue;
use crate::stmt::{Block, Expression, Print, Stmt, Var};
use crate::token::Token;
use crate::tokentype::TokenType;

pub struct Parser {
    // lox: &'a mut Lox,
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            // lox,
            tokens,
            current: 0,
        }
    }

    pub(crate) fn parse(&mut self) -> (Vec<Box<dyn Stmt>>, Vec<(Token, String)>) {
        let mut statements: Vec<Box<dyn Stmt>> = Vec::new();
        let mut errors: Vec<(Token, String)> = Vec::new();
        while !self.is_at_end() {
            match self.declaration() {
                Ok(statement) => statements.push(statement),
                Err((msg, token)) => errors.push((token.clone(), String::from(msg))),
            }
        }
        (statements, errors)
    }

    fn expression(&mut self) -> Result<Box<dyn Expr>, (&'static str, Token)> {
        self.assignment()
    }

    fn declaration(&mut self) -> Result<Box<dyn Stmt>, (&'static str, Token)> {
        if self.matching(&[TokenType::Var]) {
            self.var_declaration()
        } else {
            let statement = self.statement();
            match statement {
                Ok(_) => statement,
                Err(e) => {
                    self.synchronize();
                    Err(e)
                }
            }
        }
    }

    fn statement(&mut self) -> Result<Box<dyn Stmt>, (&'static str, Token)> {
        if self.matching(&[TokenType::Print]) {
            return self.print_statement();
        }

        if self.matching(&[TokenType::LeftBrace]) {
            let statements = self.block()?;
            return Ok(Box::new(Block { statements }));
        }

        self.expression_statement()
    }

    fn print_statement(&mut self) -> Result<Box<dyn Stmt>, (&'static str, Token)> {
        let expression = self.expression()?;
        let consumed = self.consume(TokenType::SemiColon, "Expect ';' after expression.");
        match consumed {
            Ok(_) => Ok(Box::new(Print { expression })),
            Err(e) => Err(e),
        }
    }

    fn var_declaration(&mut self) -> Result<Box<dyn Stmt>, (&'static str, Token)> {
        //TODO hmm used without ending with ; consume semi colon somehow
        let name = self
            .consume(TokenType::Identifier, "Expect variable name.")?
            .clone();
        if self.matching(&[TokenType::Equal]) {
            let initializer = self.expression()?;
            Ok(Box::new(Var { name, initializer }))
        } else {
            Ok(Box::new(Var {
                name,
                initializer: Box::new(NoOp {}),
            }))
        }
    }

    fn expression_statement(&mut self) -> Result<Box<dyn Stmt>, (&'static str, Token)> {
        let expression = self.expression()?;
        let consumed = self.consume(TokenType::SemiColon, "Expect ';' after expression.");
        match consumed {
            Ok(_) => Ok(Box::new(Expression { expression })),
            Err(e) => Err(e),
        }
    }

    fn block(&mut self) -> Result<Vec<Box<dyn Stmt>>, (&'static str, Token)> {
        let mut statements: Vec<Box<dyn Stmt>> = Vec::new();

        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?)
        }

        self.consume(TokenType::RightBrace, "Expect '}' after block.")?;
        Ok(statements)
    }

    fn assignment(&mut self) -> Result<Box<dyn Expr>, (&'static str, Token)> {
        let expr = self.equality()?;
        if self.matching(&[TokenType::Equal]) {
            let equals = self.previous().clone();
            let value = self.assignment()?;

            match expr.kind() {
                Kind::Variable(name) => Ok(Box::new(Assign { name, value })),
                _ => {
                    const MSG: &str = "Invalid assignment target.";
                    // self.error(&equals, MSG);
                    Err((MSG, equals))
                }
            }
        } else {
            Ok(expr)
        }
    }

    fn equality(&mut self) -> Result<Box<dyn Expr>, (&'static str, Token)> {
        let mut expr = self.comparison()?;
        let mut matching = self.matching(&[TokenType::BangEqual, TokenType::EqualEqual]);
        while matching {
            let operator = self.previous().clone();
            let right = self.comparison()?;
            expr = Box::new(Binary {
                left: expr,
                operator,
                right,
            });
            matching = self.matching(&[TokenType::BangEqual, TokenType::EqualEqual]);
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Box<dyn Expr>, (&'static str, Token)> {
        let mut expr = self.term()?;
        let types = &[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ];
        let mut matching = self.matching(types);
        while matching {
            let operator = self.previous().clone();
            let right = self.term()?;
            expr = Box::new(Binary {
                left: expr,
                operator,
                right,
            });
            matching = self.matching(types);
        }
        Ok(expr)
    }

    fn term(&mut self) -> Result<Box<dyn Expr>, (&'static str, Token)> {
        let mut expr = self.factor()?;
        let types = &[TokenType::Minus, TokenType::Plus];
        let mut matching = self.matching(types);
        while matching {
            let operator = self.previous().clone();
            let right = self.factor()?;
            expr = Box::new(Binary {
                left: expr,
                operator,
                right,
            });
            matching = self.matching(types);
        }
        Ok(expr)
    }

    fn factor(&mut self) -> Result<Box<dyn Expr>, (&'static str, Token)> {
        let mut expr = self.unary()?;
        let types = &[TokenType::Slash, TokenType::Star];
        let mut matching = self.matching(types);
        while matching {
            let operator = self.previous().clone();
            let right = self.unary()?;
            expr = Box::new(Binary {
                left: expr,
                operator,
                right,
            });
            matching = self.matching(types);
        }
        Ok(expr)
    }

    fn unary(&mut self) -> Result<Box<dyn Expr>, (&'static str, Token)> {
        let types = &[TokenType::Minus, TokenType::Bang];
        let matching = self.matching(types);
        if matching {
            let operator = self.previous().clone();
            let right = self.unary()?;
            let expr = Box::new(Unary { operator, right });
            return Ok(expr);
        }
        self.primary()
    }

    fn primary(&mut self) -> Result<Box<dyn Expr>, (&'static str, Token)> {
        if self.matching(&[TokenType::False]) {
            return Ok(Box::new(Literal {
                value: LoxValue::Bool(false),
            }));
        }

        if self.matching(&[TokenType::True]) {
            return Ok(Box::new(Literal {
                value: LoxValue::Bool(true),
            }));
        }

        if self.matching(&[TokenType::Nil]) {
            return Ok(Box::new(Literal {
                value: LoxValue::None,
            }));
        }

        if self.matching(&[TokenType::String, TokenType::Number]) {
            return Ok(Box::new(Literal {
                value: self.previous().literal.clone(),
            }));
        }

        if self.matching(&[TokenType::Identifier]) {
            return Ok(Box::new(Variable {
                name: self.previous().clone(),
            }));
        }

        if self.matching(&[TokenType::LeftParen]) {
            let expression = self.expression()?;
            self.consume(TokenType::RightParen, "Expect ')' after expression.")?;
            return Ok(Box::new(Grouping { expression }));
        }

        Ok(Box::new(NoOp {}))
    }

    fn matching(&mut self, types: &[TokenType]) -> bool {
        for ttype in types {
            if self.check(ttype.clone()) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn consume(
        &mut self,
        ttype: TokenType,
        msg: &'static str,
    ) -> Result<&Token, (&'static str, Token)> {
        if self.check(ttype) {
            Ok(self.advance())
        } else {
            Err((msg, self.peek().clone()))
        }
    }

    fn check(&self, ttype: TokenType) -> bool {
        !self.is_at_end() && (self.peek().token_type == ttype)
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current = self.current + 1;
        }

        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::EOF
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    // fn error(&mut self, token: &Token, msg: &'static str) -> Result<&Token, &'static str> {
    //     self.lox.error_parse(token, msg);
    //     Err(msg)
    // }

    fn synchronize(&mut self) {
        self.advance();
        while !self.is_at_end() {
            if self.previous().token_type == TokenType::SemiColon {
                return;
            }

            match self.peek().token_type {
                TokenType::Class
                | TokenType::For
                | TokenType::Fun
                | TokenType::If
                | TokenType::Print
                | TokenType::Return
                | TokenType::Var
                | TokenType::While => return,
                _ => {}
            }

            self.advance();
        }
    }
}
