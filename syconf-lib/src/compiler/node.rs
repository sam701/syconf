use crate::compiler::*;

use super::context::Context;
use super::methods;
use super::value::{UserDefinedFunction, Value};
use crate::compiler::value::Func;

#[derive(Debug)]
pub struct FunctionDefinition {
    pub argument_names: Option<Vec<String>>,
    pub node: CodeNode,
}

#[derive(Debug)]
pub enum NodeContent {
    Resolved(Value),

    List(Vec<CodeNode>),
    HashMap(Vec<HmEntry>),

    FunctionDefinition(Rc<FunctionDefinition>),

    FunctionInputArgument(String),
    FunctionCall {
        name: String,
        function: CodeNode,

        // If arguments is None, it is just a variable, i.e. the value as it is.
        arguments: Option<Vec<CodeNode>>,
    },
}

#[derive(Debug)]
pub struct HmEntry {
    pub key: CodeNode,
    pub value: CodeNode,
}

#[derive(Clone, Derivative)]
#[derivative(Debug = "transparent")]
pub struct CodeNode(Rc<CodeNodeRef>);

#[derive(Derivative)]
#[derivative(Debug)]
struct CodeNodeRef {
    #[derivative(Debug = "ignore")]
    location: Option<Location>,
    content: NodeContent,
}

impl CodeNode {
    pub fn new(content: NodeContent, location: Option<Location>) -> Self {
        Self(Rc::new(CodeNodeRef {
            content,
            location,
        }))
    }

    pub fn resolve(&self, ctx: &Context) -> Result<Value, Error> {
        match &self.0.content {
            NodeContent::Resolved(v) => Ok(v.clone()),
            NodeContent::FunctionInputArgument(name) =>
                ctx.get_value(name)
                    .ok_or(anyhow!("Function argument '{}' is not bound", name))
                    .and_then(|x| x.resolve(ctx)),
            NodeContent::FunctionDefinition(fd) =>
                Ok(Value::Func(Func::new_user_defined(ctx.clone(), fd.clone()))),
            NodeContent::List(list) => list.iter()
                .map(|x| x.resolve(ctx))
                .collect::<Result<Vec<Value>, Error>>()
                .map(Rc::new)
                .map(Value::List),
            NodeContent::HashMap(hm) => hm.iter()
                .map(|HmEntry{key, value}| Ok((key.resolve(ctx)?.as_str()?.to_string(), value.resolve(ctx)?)))
                .collect::<Result<HashMap<String, Value>, Error>>()
                .map(Rc::new)
                .map(Value::HashMap),
            NodeContent::FunctionCall { name: _, function, arguments } => {
                let opt_args = arguments.as_ref().map(|x|
                    x.iter().map(|en| en.resolve(ctx)).collect::<Result<Vec<Value>, Error>>())
                    .map_or(Ok(None), |v| v.map(|x| Some(x)))?;
                match (&function.resolve(ctx)?, &opt_args) {
                    (Value::Func(func), Some(args)) => func.call(args.as_slice()),
                    (x, Some(_)) => Err(anyhow!("value is not a function")),
                    (x, None) => Ok(x.clone()),
                }
            }
        }
    }
}
