use crate::expr::{
    Assign, Binary, Call, Expr, Get, Grouping, Kind, Literal, Logical, NoOp, Set, This, Unary,
    Variable,
};
use crate::loxvalue::LoxValue;
use crate::stmt::{
    Block, ClassStmt, Expression, Function, If, Print, ReturnStmt, Stmt, Var, While,
};
use crate::token::Token;
use crate::tokentype::TokenType;
use std::rc::Rc;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    in_a_class: bool,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            current: 0,
            in_a_class: false,
        }
    }

    pub(crate) fn parse(&mut self) -> (Vec<Rc<dyn Stmt>>, Vec<(Token, String)>) {
        let mut statements: Vec<Rc<dyn Stmt>> = Vec::new();
        let mut errors: Vec<(Token, String)> = Vec::new();
        while !self.is_at_end() {
            match self.declaration() {
                Ok(statement) => statements.push(statement),
                Err((msg, token)) => errors.push((token.clone(), msg)),
            }
        }
        (statements, errors)
    }

    fn expression(&mut self) -> Result<Rc<dyn Expr>, (String, Token)> {
        self.assignment()
    }

    fn declaration(&mut self) -> Result<Rc<dyn Stmt>, (String, Token)> {
        if self.matching(&[TokenType::Class]) {
            self.class_declaration()
        } else if self.matching(&[TokenType::Fun]) {
            self.function("function")
        } else if self.matching(&[TokenType::Var]) {
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

    fn class_declaration(&mut self) -> Result<Rc<dyn Stmt>, (String, Token)> {
        self.in_a_class = true;
        let name = self
            .consume(TokenType::Identifier, String::from("Expect class name."))?
            .clone();
        self.consume(
            TokenType::LeftBrace,
            String::from("Expect '{' before class body"),
        )?;
        let mut methods: Vec<Rc<dyn Stmt>> = Vec::new();

        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            methods.push(self.function("method")?);
        }

        self.consume(
            TokenType::RightBrace,
            String::from("Expect '}' after class body"),
        )?;
        self.in_a_class = false;
        Ok(Rc::new(ClassStmt { name, methods }))
    }

    fn statement(&mut self) -> Result<Rc<dyn Stmt>, (String, Token)> {
        if self.matching(&[TokenType::For]) {
            return self.for_statement();
        }
        if self.matching(&[TokenType::If]) {
            return self.if_statement();
        }
        if self.matching(&[TokenType::Print]) {
            return self.print_statement();
        }
        if self.matching(&[TokenType::Return]) {
            return self.return_statement();
        }
        if self.matching(&[TokenType::While]) {
            return self.while_statement();
        }

        if self.matching(&[TokenType::LeftBrace]) {
            let statements = self.block()?;
            return Ok(Rc::new(Block { statements }));
        }

        self.expression_statement()
    }

    fn for_statement(&mut self) -> Result<Rc<dyn Stmt>, (String, Token)> {
        self.consume(
            TokenType::LeftParen,
            String::from("Expect '(' after 'for'."),
        )?;
        let initializer: Option<Rc<dyn Stmt>> = if self.matching(&[TokenType::SemiColon]) {
            None
        } else if self.matching(&[TokenType::Var]) {
            Some(self.var_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };

        let condition: Option<Rc<dyn Expr>> = if !self.check(TokenType::SemiColon) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(
            TokenType::SemiColon,
            String::from("Expect ';' after loop condition."),
        )?;

        let increment: Option<Rc<dyn Expr>> = if !self.check(TokenType::RightParen) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(
            TokenType::RightParen,
            String::from("Expect ')' after for clauses."),
        )?;

        let mut body = self.statement()?;

        match increment {
            Some(a) => {
                body = Rc::new(Block {
                    statements: vec![body, Rc::new(Expression { expression: a })],
                })
            }
            None => {}
        }

        let condition_result = match condition {
            None => Rc::new(Literal {
                value: LoxValue::Bool(true),
            }),
            Some(a) => a,
        };

        body = Rc::new(While {
            condition: condition_result,
            body,
        });

        match initializer {
            None => {}
            Some(a) => {
                body = Rc::new(Block {
                    statements: vec![a, body],
                })
            }
        }

        Ok(body)
    }

    fn if_statement(&mut self) -> Result<Rc<dyn Stmt>, (String, Token)> {
        self.consume(TokenType::LeftParen, String::from("Expect '(' after 'if'."))?;
        let condition = self.expression()?;
        self.consume(
            TokenType::RightParen,
            String::from("Expect ')' after if condition."),
        )?;

        let then_branch = self.statement()?;
        let mut else_branch = None;

        if self.matching(&[TokenType::Else]) {
            else_branch = Some(self.statement()?);
        }

        Ok(Rc::new(If {
            condition,
            then_branch,
            else_branch,
        }))
    }

    fn print_statement(&mut self) -> Result<Rc<dyn Stmt>, (String, Token)> {
        let expression = self.expression()?;
        let consumed = self.consume(
            TokenType::SemiColon,
            String::from("Expect ';' after expression."),
        );
        match consumed {
            Ok(_) => Ok(Rc::new(Print { expression })),
            Err(e) => Err(e),
        }
    }

    fn return_statement(&mut self) -> Result<Rc<dyn Stmt>, (String, Token)> {
        let keyword = self.previous().clone();
        let value = if !self.check(TokenType::SemiColon) {
            self.expression()?
        } else {
            Rc::new(NoOp {})
        };
        self.consume(
            TokenType::SemiColon,
            String::from("Expect ';' after return value."),
        )?;
        Ok(Rc::new(ReturnStmt { keyword, value }))
    }

    fn var_declaration(&mut self) -> Result<Rc<dyn Stmt>, (String, Token)> {
        let name = self
            .consume(TokenType::Identifier, String::from("Expect variable name."))?
            .clone();
        let to_return: Result<Rc<dyn Stmt>, (String, Token)> = if self.matching(&[TokenType::Equal])
        {
            let initializer = self.expression()?;
            Ok(Rc::new(Var { name, initializer }))
        } else {
            Ok(Rc::new(Var {
                name,
                initializer: Rc::new(NoOp {}),
            }))
        };
        self.consume(
            TokenType::SemiColon,
            String::from("Expect ';' after var declaration."),
        )?;
        to_return
    }

    fn while_statement(&mut self) -> Result<Rc<dyn Stmt>, (String, Token)> {
        self.consume(
            TokenType::LeftParen,
            String::from("Expect '(' after while."),
        )?;
        let condition = self.expression()?;
        self.consume(
            TokenType::RightParen,
            String::from("Expect ')' after condition."),
        )?;
        let body = self.statement()?;
        Ok(Rc::new(While { condition, body }))
    }

    fn expression_statement(&mut self) -> Result<Rc<dyn Stmt>, (String, Token)> {
        let expression = self.expression()?;
        let consumed = self.consume(
            TokenType::SemiColon,
            String::from("Expect ';' after expression."),
        );
        match consumed {
            Ok(_) => Ok(Rc::new(Expression { expression })),
            Err(e) => Err(e),
        }
    }

    fn function(&mut self, kind: &'static str) -> Result<Rc<dyn Stmt>, (String, Token)> {
        let name = self
            .consume(TokenType::Identifier, format!("Expect {} name.", kind))?
            .clone();
        self.consume(
            TokenType::LeftParen,
            format!("Expect '(' after {} name.", kind),
        )?;
        let mut parameters: Vec<Token> = Vec::new();
        if !self.check(TokenType::RightParen) {
            parameters.push(
                self.consume(
                    TokenType::Identifier,
                    String::from("Expect parameter name."),
                )?
                .clone(),
            );
            while self.matching(&[TokenType::Comma]) {
                if parameters.len() >= 255 {
                    return Err((
                        String::from("Can't have more than 255 parameters."),
                        self.peek().clone(),
                    ));
                }
                parameters.push(
                    self.consume(
                        TokenType::Identifier,
                        String::from("Expect parameter name."),
                    )?
                    .clone(),
                );
            }
        }
        self.consume(
            TokenType::RightParen,
            String::from("Expect ')' after parameters."),
        )?;
        self.consume(
            TokenType::LeftBrace,
            format!("Expect '{{' before {} body.", kind),
        )?;
        let body = self.block()?;
        Ok(Rc::new(Function {
            name,
            params: parameters.clone(),
            body,
        }))
    }

    fn block(&mut self) -> Result<Vec<Rc<dyn Stmt>>, (String, Token)> {
        let mut statements: Vec<Rc<dyn Stmt>> = Vec::new();

        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?)
        }

        self.consume(
            TokenType::RightBrace,
            String::from("Expect '}' after block."),
        )?;
        Ok(statements)
    }

    fn assignment(&mut self) -> Result<Rc<dyn Expr>, (String, Token)> {
        let expr = self.or()?;
        if self.matching(&[TokenType::Equal]) {
            let equals = self.previous().clone();
            let value = self.assignment()?;

            match expr.kind() {
                Kind::Variable(name) => Ok(Rc::new(Assign { name, value })),
                Kind::Get(name, object) => Ok(Rc::new(Set {
                    object,
                    name,
                    value,
                })),
                _ => {
                    let msg: String = String::from("Invalid assignment target.");
                    // self.error(&equals, MSG);
                    Err((msg, equals))
                }
            }
        } else {
            Ok(expr)
        }
    }

    fn or(&mut self) -> Result<Rc<dyn Expr>, (String, Token)> {
        let mut expr = self.and()?;

        while self.matching(&[TokenType::Or]) {
            let operator = self.previous().clone();
            let right = self.and()?;
            expr = Rc::new(Logical {
                left: expr,
                operator,
                right,
            })
        }
        Ok(expr)
    }

    fn and(&mut self) -> Result<Rc<dyn Expr>, (String, Token)> {
        let mut expr = self.equality()?;
        while self.matching(&[TokenType::And]) {
            let operator = self.previous().clone();
            let right = self.equality()?;
            expr = Rc::new(Logical {
                left: expr,
                operator,
                right,
            })
        }
        Ok(expr)
    }

    fn equality(&mut self) -> Result<Rc<dyn Expr>, (String, Token)> {
        let mut expr = self.comparison()?;
        let mut matching = self.matching(&[TokenType::BangEqual, TokenType::EqualEqual]);
        while matching {
            let operator = self.previous().clone();
            let right = self.comparison()?;
            expr = Rc::new(Binary {
                left: expr,
                operator,
                right,
            });
            matching = self.matching(&[TokenType::BangEqual, TokenType::EqualEqual]);
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Rc<dyn Expr>, (String, Token)> {
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
            expr = Rc::new(Binary {
                left: expr,
                operator,
                right,
            });
            matching = self.matching(types);
        }
        Ok(expr)
    }

    fn term(&mut self) -> Result<Rc<dyn Expr>, (String, Token)> {
        let mut expr = self.factor()?;
        let types = &[TokenType::Minus, TokenType::Plus];
        let mut matching = self.matching(types);
        while matching {
            let operator = self.previous().clone();
            let right = self.factor()?;
            expr = Rc::new(Binary {
                left: expr,
                operator,
                right,
            });
            matching = self.matching(types);
        }
        Ok(expr)
    }

    fn factor(&mut self) -> Result<Rc<dyn Expr>, (String, Token)> {
        let mut expr = self.unary()?;
        let types = &[TokenType::Slash, TokenType::Star];
        let mut matching = self.matching(types);
        while matching {
            let operator = self.previous().clone();
            let right = self.unary()?;
            expr = Rc::new(Binary {
                left: expr,
                operator,
                right,
            });
            matching = self.matching(types);
        }
        Ok(expr)
    }

    fn unary(&mut self) -> Result<Rc<dyn Expr>, (String, Token)> {
        let types = &[TokenType::Minus, TokenType::Bang];
        let matching = self.matching(types);
        if matching {
            let operator = self.previous().clone();
            let right = self.unary()?;
            let expr = Rc::new(Unary { operator, right });
            return Ok(expr);
        }
        self.call()
    }

    fn finish_call(&mut self, callee: Rc<dyn Expr>) -> Result<Rc<dyn Expr>, (String, Token)> {
        let mut arguments: Vec<Rc<dyn Expr>> = Vec::new();
        if !self.check(TokenType::RightParen) {
            arguments.push(self.expression()?);
            while self.matching(&[TokenType::Comma]) {
                if arguments.len() >= 255 {
                    return Err((
                        String::from("Can't have more than 255 arguments"),
                        self.peek().clone(),
                    ));
                }
                arguments.push(self.expression()?);
            }
        }

        let paren = self
            .consume(
                TokenType::RightParen,
                String::from("Expect ')' after arguments."),
            )?
            .clone();
        Ok(Rc::new(Call {
            callee,
            paren,
            arguments,
        }))
    }

    fn call(&mut self) -> Result<Rc<dyn Expr>, (String, Token)> {
        let mut expr = self.primary()?;
        loop {
            if self.matching(&[TokenType::LeftParen]) {
                expr = self.finish_call(expr)?;
            } else if self.matching(&[TokenType::Dot]) {
                let name = self
                    .consume(
                        TokenType::Identifier,
                        String::from("Expect property  name after '.'."),
                    )?
                    .clone();
                expr = Rc::new(Get {
                    name,
                    object: Rc::clone(&expr),
                })
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn primary(&mut self) -> Result<Rc<dyn Expr>, (String, Token)> {
        if self.matching(&[TokenType::False]) {
            return Ok(Rc::new(Literal {
                value: LoxValue::Bool(false),
            }));
        }

        if self.matching(&[TokenType::True]) {
            return Ok(Rc::new(Literal {
                value: LoxValue::Bool(true),
            }));
        }

        if self.matching(&[TokenType::Nil]) {
            return Ok(Rc::new(Literal {
                value: LoxValue::None,
            }));
        }

        if self.matching(&[TokenType::String, TokenType::Number]) {
            return Ok(Rc::new(Literal {
                value: self.previous().literal.clone(),
            }));
        }

        if self.matching(&[TokenType::This]) {
            return if self.in_a_class {
                Ok(Rc::new(This {
                    keyword: self.previous().clone(),
                }))
            } else {
                return Err((
                    String::from("Can't use 'this' outside of a class"),
                    self.peek().clone(),
                ));
            };
        }

        if self.matching(&[TokenType::Identifier]) {
            return Ok(Rc::new(Variable {
                name: self.previous().clone(),
            }));
        }

        if self.matching(&[TokenType::LeftParen]) {
            let expression = self.expression()?;
            self.consume(
                TokenType::RightParen,
                String::from("Expect ')' after expression."),
            )?;
            return Ok(Rc::new(Grouping { expression }));
        }

        Ok(Rc::new(NoOp {}))
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

    fn consume(&mut self, ttype: TokenType, msg: String) -> Result<&Token, (String, Token)> {
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
        //TODO out of bounds
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
