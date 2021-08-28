use crate::expr::{Assign, Binary, Expr, Grouping, Kind, Literal, Logical, NoOp, Unary, Variable};
use crate::loxvalue::LoxValue;
use crate::stmt::{Block, Expression, If, Print, Stmt, Var, While};
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
        if self.matching(&[TokenType::For]) {
            return self.for_statement();
        }
        if self.matching(&[TokenType::If]) {
            return self.if_statement();
        }
        if self.matching(&[TokenType::Print]) {
            return self.print_statement();
        }
        if self.matching(&[TokenType::While]) {
            return self.while_statement();
        }

        if self.matching(&[TokenType::LeftBrace]) {
            let statements = self.block()?;
            return Ok(Box::new(Block { statements }));
        }

        self.expression_statement()
    }

    fn for_statement(&mut self) -> Result<Box<dyn Stmt>, (&'static str, Token)> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'for'.")?;
        let initializer: Option<Box<dyn Stmt>> = if self.matching(&[TokenType::SemiColon]) {
            None
        } else if self.matching(&[TokenType::Var]) {
            Some(self.var_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };

        let condition: Option<Box<dyn Expr>> = if !self.check(TokenType::SemiColon) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(TokenType::SemiColon, "Expect ';' after loop condition.")?;

        let increment: Option<Box<dyn Expr>> = if !self.check(TokenType::RightParen) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(TokenType::RightParen, "Expect ')' after for clauses.")?;

        let mut body = self.statement()?;

        match increment {
            Some(a) => {
                body = Box::new(Block {
                    statements: vec![body, Box::new(Expression { expression: a })],
                })
            }
            None => {}
        }

        let condition_result = match condition {
            None => Box::new(Literal {
                value: LoxValue::Bool(true),
            }),
            Some(a) => a,
        };

        body = Box::new(While {
            condition: condition_result,
            body,
        });

        match initializer {
            None => {}
            Some(a) => {
                body = Box::new(Block {
                    statements: vec![a, body],
                })
            }
        }

        Ok(body)
    }

    fn if_statement(&mut self) -> Result<Box<dyn Stmt>, (&'static str, Token)> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after if condition.")?;

        let then_branch = self.statement()?;
        let mut else_branch = None;

        if self.matching(&[TokenType::Else]) {
            else_branch = Some(self.statement()?);
        }

        Ok(Box::new(If {
            condition,
            then_branch,
            else_branch,
        }))
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
        let name = self
            .consume(TokenType::Identifier, "Expect variable name.")?
            .clone();
        let to_return: Result<Box<dyn Stmt>, (&'static str, Token)> =
            if self.matching(&[TokenType::Equal]) {
                let initializer = self.expression()?;
                Ok(Box::new(Var { name, initializer }))
            } else {
                Ok(Box::new(Var {
                    name,
                    initializer: Box::new(NoOp {}),
                }))
            };
        self.consume(TokenType::SemiColon, "Expect ';' after var declaration.")?;
        to_return
    }

    fn while_statement(&mut self) -> Result<Box<dyn Stmt>, (&'static str, Token)> {
        self.consume(TokenType::LeftParen, "Expect '(' after while.")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after condition.")?;
        let body = self.statement()?;
        Ok(Box::new(While { condition, body }))
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
        let expr = self.or()?;
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

    fn or(&mut self) -> Result<Box<dyn Expr>, (&'static str, Token)> {
        let mut expr = self.and()?;

        while self.matching(&[TokenType::Or]) {
            let operator = self.previous().clone();
            let right = self.and()?;
            expr = Box::new(Logical {
                left: expr,
                operator,
                right,
            })
        }
        Ok(expr)
    }

    fn and(&mut self) -> Result<Box<dyn Expr>, (&'static str, Token)> {
        let mut expr = self.equality()?;
        while self.matching(&[TokenType::And]) {
            let operator = self.previous().clone();
            let right = self.equality()?;
            expr = Box::new(Logical {
                left: expr,
                operator,
                right,
            })
        }
        Ok(expr)
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
