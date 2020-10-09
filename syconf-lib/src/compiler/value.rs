use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use crate::compiler::context::Context;
use crate::compiler::Error;
use crate::compiler::node::{FunctionDefinition, NodeContent};
use crate::parser::FuncDefinition;

use super::node::CodeNode;

#[derive(Clone, Debug, PartialEq, serde::Deserialize)]
#[serde(untagged)]
pub enum Value {
    Bool(bool),
    Int(i32),
    String(Rc<String>),
    HashMap(Rc<HashMap<String, Value>>),
    List(Rc<Vec<Value>>),
    #[serde(skip_deserializing)]
    Func(Func),
}

#[derive(thiserror::Error, Debug)]
#[error("Type Mismatch: expects {} but was {}", .expects, .was)]
pub struct TypeMismatch {
    expects: String,
    was: String,
}

impl Value {
    fn fail(&self, expected: &str) -> TypeMismatch {
        TypeMismatch{
            expects: expected.to_string(),
            was: format!("{:?}", self),
        }
    }

    pub fn as_int(&self) -> Result<i32, TypeMismatch> {
        if let Value::Int(x) = self {
            Ok(*x)
        } else {
            Err(self.fail("int"))
        }
    }
    pub fn as_str(&self) -> Result<&str, TypeMismatch> {
        if let Value::String(x) = self {
            Ok(x.as_str())
        } else {
            Err(self.fail("string"))
        }
    }
    pub fn as_bool(&self) -> Result<bool, TypeMismatch> {
        if let Value::Bool(x) = self {
            Ok(*x)
        } else {
            Err(self.fail("bool"))
        }
    }
    pub fn as_list(&self) -> Result<&Vec<Value>, TypeMismatch> {
        if let Value::List(x) = self {
            Ok(x)
        } else {
            Err(self.fail("list"))
        }
    }
    pub fn as_hashmap(&self) -> Result<&HashMap<String, Value>, TypeMismatch> {
        if let Value::HashMap(x) = self {
            Ok(x)
        } else {
            Err(self.fail("hashmap"))
        }
    }
    pub fn as_func(&self) -> Result<Func, TypeMismatch> {
        if let Value::Func(func) = self {
            Ok(func.clone())
        } else {
            Err(self.fail("function"))
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a.partial_cmp(b),
            (Value::String(a), Value::String(b)) => a.partial_cmp(b),
            (Value::Bool(a), Value::Bool(b)) => a.partial_cmp(b),
            _ => None
        }
    }
}

#[derive(Clone)]
pub struct Func(FuncInner);

impl PartialEq for Func {
    fn eq(&self, other: &Self) -> bool {
        false
    }
}

impl fmt::Debug for Func {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            FuncInner::BuiltInFunction(_) => f.write_str("<func>"),
            FuncInner::BuiltInMethod(_) => f.write_str("<func>"),
            FuncInner::UserDefined(ud) => f.write_str(format!("user_func:{:?}", ud.definition.as_ref()).as_str()),
        }
    }
}

impl Func {
    pub fn new_builtin(func: &'static (dyn Fn(&[Value]) -> Result<Value, Error>)) -> Self {
        Self(FuncInner::BuiltInFunction(func))
    }

    pub fn new_method(method: Method) -> Self {
        Self(FuncInner::BuiltInMethod(method))
    }

    pub fn new_user_defined(context: Context, definition: Rc<FunctionDefinition>) -> Self {
        Self(FuncInner::UserDefined(UserDefinedFunction {
            context,
            definition,
        }))
    }

    pub fn call(&self, args: &[Value]) -> Result<Value, Error> {
        match &self.0 {
            FuncInner::BuiltInFunction(func) => func(args),
            FuncInner::BuiltInMethod(method) => method.call(args),
            FuncInner::UserDefined(ud) => ud.call(args),
        }
    }
}

#[derive(Clone)]
enum FuncInner {
    BuiltInFunction(&'static dyn Fn(&[Value]) -> Result<Value, Error>),
    BuiltInMethod(Method),
    UserDefined(UserDefinedFunction),
}

#[derive(Clone)]
pub enum Method {
    HashMap(Rc<HashMap<String, Value>>, &'static dyn Fn(&HashMap<String, Value>, &[Value]) -> Result<Value, Error>),
    List(Rc<Vec<Value>>, &'static dyn Fn(&Vec<Value>, &[Value]) -> Result<Value, Error>),
    String(Rc<String>, &'static dyn Fn(&String, &[Value]) -> Result<Value, Error>),
}

impl Method {
    fn call(&self, args: &[Value]) -> Result<Value, Error> {
        match self {
            Method::HashMap(hm, func) => func(hm, args),
            Method::List(list, func) => func(list, args),
            Method::String(string, func) => func(string, args),
        }
    }
}


#[derive(Debug, Clone)]
pub struct UserDefinedFunction {
    context: Context,
    definition: Rc<FunctionDefinition>,
}

impl UserDefinedFunction {
    fn call(&self, args: &[Value]) -> Result<Value, Error> {
        debug!(arg_names=?self.definition.argument_names, input=?args, "applying user defined function");
        debug!(node=?self.definition.node, "user defined");
        let nctx = self.context.new_child();
        // TODO: check args
        let arg_names = self.definition.argument_names.as_ref().unwrap();
        for ix in 0..arg_names.len() {
            nctx.bind(
                arg_names[ix].clone(),
                CodeNode::new(NodeContent::Resolved(args[ix].clone()), None),
            );
        }
        self.definition.node.resolve(&nctx)
    }
}