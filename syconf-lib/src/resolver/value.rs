use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::sync::Arc;

use crate::resolver::context::Context;
use crate::resolver::methods::hashmap::HashmapMethod;
use crate::resolver::methods::list::ListMethod;
use crate::resolver::methods::string::StringMethod;
use crate::resolver::node::{FunctionDefinition, NodeContent};
use crate::resolver::{Error, ErrorWithLocation};

use super::node::CodeNode;
use crate::parser::number::Number;

pub type ValueString = Arc<str>;

#[derive(Clone, Debug, PartialEq, serde::Deserialize)]
#[serde(untagged)]
pub enum Value {
    Bool(bool),
    Number(Number),
    String(ValueString),
    HashMap(Arc<HashMap<ValueString, Value>>),
    List(Arc<[Value]>),
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
        TypeMismatch {
            expects: expected.to_string(),
            was: format!("{:?}", self),
        }
    }

    pub fn as_int(&self) -> Result<i64, TypeMismatch> {
        if let Value::Number(Number::Int(x)) = self {
            Ok(*x)
        } else {
            Err(self.fail("int"))
        }
    }
    pub fn as_float(&self) -> Result<f64, TypeMismatch> {
        if let Value::Number(Number::Float(x)) = self {
            Ok(*x)
        } else {
            Err(self.fail("float"))
        }
    }
    pub fn as_value_string(&self) -> Result<&ValueString, TypeMismatch> {
        if let Value::String(x) = self {
            Ok(x)
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
    pub fn as_list(&self) -> Result<&[Value], TypeMismatch> {
        if let Value::List(x) = self {
            Ok(x)
        } else {
            Err(self.fail("list"))
        }
    }
    pub fn as_hashmap(&self) -> Result<&HashMap<ValueString, Value>, TypeMismatch> {
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

    pub fn to_serializable(&self) -> SerializableValue {
        match self {
            Value::Bool(x) => SerializableValue::Bool(*x),
            Value::Number(x) => SerializableValue::Number(x.clone()),
            Value::String(x) => SerializableValue::String(x.clone()),
            Value::HashMap(x) => SerializableValue::HashMap(
                x.iter()
                    .map(|(k, v)| (k.clone(), v.to_serializable()))
                    .collect(),
            ),
            Value::List(x) => {
                SerializableValue::List(x.iter().map(Value::to_serializable).collect())
            }
            Value::Func(_) => SerializableValue::String("<function>".into()),
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Value::Number(Number::Int(a)), Value::Number(Number::Int(b))) => a.partial_cmp(b),
            (Value::Number(Number::Float(a)), Value::Number(Number::Float(b))) => a.partial_cmp(b),
            (Value::String(a), Value::String(b)) => a.partial_cmp(b),
            (Value::Bool(a), Value::Bool(b)) => a.partial_cmp(b),
            _ => None,
        }
    }
}

#[derive(Clone)]
pub struct Func(FuncInner);

impl PartialEq for Func {
    fn eq(&self, _: &Self) -> bool {
        false
    }
}

impl fmt::Debug for Func {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            FuncInner::BuiltInFunction(_) => f.write_str("<func>"),
            FuncInner::BuiltInMethod(_) => f.write_str("<func>"),
            FuncInner::UserDefined(ud) => {
                f.write_str(format!("user_func:{:?}", ud.definition.as_ref()).as_str())
            }
        }
    }
}

impl Func {
    pub fn new_builtin(func: &'static FunctionSig) -> Self {
        Self(FuncInner::BuiltInFunction(func))
    }

    pub fn new_method(method: Method) -> Self {
        Self(FuncInner::BuiltInMethod(method))
    }

    pub fn new_user_defined(context: Context, definition: Arc<FunctionDefinition>) -> Self {
        Self(FuncInner::UserDefined(UserDefinedFunction {
            context,
            definition,
        }))
    }

    pub fn call(&self, args: &[Value]) -> Result<Value, ErrorWithLocation> {
        match &self.0 {
            FuncInner::BuiltInFunction(func) => func(args),
            FuncInner::BuiltInMethod(method) => method.call(args),
            FuncInner::UserDefined(ud) => ud.call(args),
        }
    }
}

pub type FunctionSig = dyn Fn(&[Value]) -> Result<Value, Error> + Send + Sync;

#[derive(Clone)]
enum FuncInner {
    BuiltInFunction(&'static FunctionSig),
    BuiltInMethod(Method),
    UserDefined(UserDefinedFunction),
}

#[derive(Clone)]
pub enum Method {
    HashMap(Arc<HashMap<ValueString, Value>>, &'static HashmapMethod),
    List(Arc<[Value]>, &'static ListMethod),
    String(ValueString, &'static StringMethod),
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
    definition: Arc<FunctionDefinition>,
}

impl UserDefinedFunction {
    fn call(&self, args: &[Value]) -> Result<Value, ErrorWithLocation> {
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

#[derive(serde::Serialize)]
#[serde(untagged)]
pub enum SerializableValue {
    Bool(bool),
    Number(Number),
    String(Arc<str>),
    HashMap(BTreeMap<Arc<str>, SerializableValue>),
    List(Arc<[SerializableValue]>),
}
