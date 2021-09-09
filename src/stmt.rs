use crate::environment::Environment;
use crate::expr::{is_truthy, Expr, Kind};
use crate::interpreter::Interpreter;
use crate::loxvalue::{Callable, Class, LoxValue};
use crate::token::Token;
use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub trait Stmt {
    fn evaluate(&self, env: Rc<Environment>) -> Result<LoxValue, (String, Token)>;
    fn kind(&self) -> StmtKind;
}

pub enum StmtKind {
    Expression,
    Print,
    Var,
    Block,
    If,
    While,
    Function(Function),
    ReturnStmt,
    ClassStmt,
}

pub struct Expression {
    pub(crate) expression: Rc<dyn Expr>,
}

impl Stmt for Expression {
    fn evaluate(&self, env: Rc<Environment>) -> Result<LoxValue, (String, Token)> {
        self.expression.evaluate(env)
    }

    fn kind(&self) -> StmtKind {
        StmtKind::Expression
    }
}

pub struct Print {
    pub(crate) expression: Rc<dyn Expr>,
}

impl Stmt for Print {
    fn evaluate(&self, env: Rc<Environment>) -> Result<LoxValue, (String, Token)> {
        match self.expression.evaluate(env) {
            Ok(value) => {
                println!("{}", value);
                Ok(LoxValue::None)
            }
            Err(e) => Err(e),
        }
    }

    fn kind(&self) -> StmtKind {
        StmtKind::Print
    }
}

pub struct Var {
    pub(crate) name: Token,
    pub(crate) initializer: Rc<dyn Expr>,
}

impl Stmt for Var {
    fn evaluate(&self, env: Rc<Environment>) -> Result<LoxValue, (String, Token)> {
        let val = self.initializer.evaluate(Rc::clone(&env))?;
        env.define(self.name.lexeme.clone(), val.clone());
        Ok(val.clone())
    }

    fn kind(&self) -> StmtKind {
        StmtKind::Var
    }
}

pub struct Block {
    pub(crate) statements: Vec<Rc<dyn Stmt>>,
}

impl Stmt for Block {
    fn evaluate(&self, env: Rc<Environment>) -> Result<LoxValue, (String, Token)> {
        let scoped_env = Rc::new(Environment::new_child(env.clone()));
        for statement in &self.statements {
            match statement.evaluate(Rc::clone(&scoped_env))? {
                LoxValue::Return(a) => {
                    return Ok(LoxValue::Return(a.clone()));
                }
                _ => {}
            }
        }
        Ok(LoxValue::None)
    }

    fn kind(&self) -> StmtKind {
        StmtKind::Block
    }
}

pub struct If {
    pub(crate) condition: Rc<dyn Expr>,
    pub(crate) then_branch: Rc<dyn Stmt>,
    pub(crate) else_branch: Option<Rc<dyn Stmt>>,
}

impl Stmt for If {
    fn evaluate(&self, env: Rc<Environment>) -> Result<LoxValue, (String, Token)> {
        match is_truthy(self.condition.evaluate(Rc::clone(&env))?, false)? {
            LoxValue::Bool(true) => self.then_branch.evaluate(Rc::clone(&env)),
            _ => match &self.else_branch {
                None => Ok(LoxValue::None),
                Some(a) => a.evaluate(Rc::clone(&env)),
            },
        }
    }

    fn kind(&self) -> StmtKind {
        StmtKind::If
    }
}

pub struct While {
    pub(crate) condition: Rc<dyn Expr>,
    pub(crate) body: Rc<dyn Stmt>,
}

impl Stmt for While {
    fn evaluate(&self, env: Rc<Environment>) -> Result<LoxValue, (String, Token)> {
        while is_truthy(self.condition.evaluate(Rc::clone(&env))?, false)? == LoxValue::Bool(true) {
            match self.body.evaluate(Rc::clone(&env))? {
                LoxValue::Return(a) => {
                    return Ok(LoxValue::Return(a.clone()));
                }
                LoxValue::None => {}
                _ => {}
            }
        }
        Ok(LoxValue::None)
    }

    fn kind(&self) -> StmtKind {
        StmtKind::While
    }
}

pub struct Function {
    pub(crate) name: Token,
    pub(crate) params: Vec<Token>,
    pub(crate) body: Vec<Rc<dyn Stmt>>,
}

impl Stmt for Function {
    fn evaluate(&self, env: Rc<Environment>) -> Result<LoxValue, (String, Token)> {
        let borrow: &Environment = env.borrow();
        let env_clone = Rc::new(borrow.clone());
        let cloned_body = self.body.clone();
        let cloned_params = self.params.clone();
        let function = LoxValue::Callable(Rc::new(Callable {
            arity: self.params.len(),
            function: Rc::new(move |arguments, environment| {
                for (i, parameter) in cloned_params.iter().enumerate() {
                    environment.define(
                        parameter.lexeme.clone(),
                        arguments.get(i).expect("Checked").clone(),
                    );
                }
                let mut interpreter = Interpreter::new_with_env(Rc::clone(&environment));
                interpreter.interpret(cloned_body.clone())
            }),
            string: format!("<fn {}>", self.name.lexeme),
            name: self.name.clone(),
            environment: Rc::clone(&env_clone),
        }));
        env.define(self.name.lexeme.clone(), function.clone());
        Ok(function)
    }

    fn kind(&self) -> StmtKind {
        StmtKind::Function(Function {
            name: self.name.clone(),
            params: self.params.clone(),
            body: self.body.clone(),
        })
    }
}

pub struct ReturnStmt {
    pub(crate) keyword: Token,
    pub(crate) value: Rc<dyn Expr>,
}

impl Stmt for ReturnStmt {
    fn evaluate(&self, env: Rc<Environment>) -> Result<LoxValue, (String, Token)> {
        match self.value.kind() {
            Kind::NoOp => Ok(LoxValue::Return(Box::new(LoxValue::None))),
            _ => Ok(LoxValue::Return(Box::new(self.value.evaluate(env)?))),
        }
    }

    fn kind(&self) -> StmtKind {
        StmtKind::ReturnStmt
    }
}

pub struct ClassStmt {
    pub(crate) name: Token,
    pub(crate) methods: Vec<Rc<dyn Stmt>>,
}

impl Stmt for ClassStmt {
    fn evaluate(&self, env: Rc<Environment>) -> Result<LoxValue, (String, Token)> {
        let mut methods: HashMap<String, LoxValue> = HashMap::new();
        for method in &self.methods {
            match method.kind() {
                StmtKind::Function(function) => {
                    let thing = function.evaluate(Rc::clone(&env))?;
                    methods.insert(function.name.lexeme.clone(), thing);
                }
                _ => {}
            }
        }
        let class = LoxValue::Class(Rc::new(Class {
            arity: 0,
            name: self.name.lexeme.clone(),
            methods: RefCell::new(methods),
        }));
        env.define(self.name.lexeme.clone(), class);
        Ok(LoxValue::None)
    }

    fn kind(&self) -> StmtKind {
        StmtKind::ClassStmt
    }
}
