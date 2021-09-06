use crate::environment::Environment;
use crate::expr::{is_truthy, Expr, Kind};
use crate::interpreter::Interpreter;
use crate::loxvalue::{Callable, LoxValue};
use crate::token::Token;
use std::rc::Rc;

pub trait Stmt {
    fn evaluate(&self, env: Rc<Environment>) -> Result<LoxValue, (String, Token)>;
}

pub struct Expression {
    pub(crate) expression: Rc<dyn Expr>,
}

impl Stmt for Expression {
    fn evaluate(&self, env: Rc<Environment>) -> Result<LoxValue, (String, Token)> {
        self.expression.evaluate(env)
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
        // update_env(env, scoped_env);
        Ok(LoxValue::None)
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
}

pub struct Function {
    pub(crate) name: Token,
    pub(crate) params: Vec<Token>,
    pub(crate) body: Vec<Rc<dyn Stmt>>,
}

impl Stmt for Function {
    fn evaluate(&self, env: Rc<Environment>) -> Result<LoxValue, (String, Token)> {
        let env_clone = Rc::clone(&env);
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
            environment: env_clone,
        }));
        env.define(self.name.lexeme.clone(), function);
        Ok(LoxValue::None)
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
}

// fn update_env(env: &mut Environment, scoped_env: Environment) -> &mut Environment {
//     match scoped_env.enclosing.clone() {
//         None => env,
//         Some(enclosing) => {
//             for (key, val) in enclosing.values.borrow().clone() {
//                 let fake_token = Token {
//                     token_type: TokenType::Var,
//                     lexeme: key,
//                     literal: LoxValue::None,
//                     line: 0,
//                 };
//                 env.assign(&fake_token, val);
//             }
//             update_env(env, *enclosing)
//         }
//     }
// }
