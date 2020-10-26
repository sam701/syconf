use crate::compiler::value::{Func, ValueString};
use crate::compiler::*;

use super::context::Context;
use super::value::Value;

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
        Self(Rc::new(CodeNodeRef { content, location }))
    }

    pub fn resolve(&self, ctx: &Context) -> Result<Value, ErrorWithLocation> {
        match &self.0.content {
            NodeContent::Resolved(v) => Ok(v.clone()),
            NodeContent::FunctionInputArgument(name) => ctx
                .get_value(name)
                .ok_or_else(|| self.err(format!("Function argument '{}' is not bound", name)))
                .and_then(|x| x.resolve(ctx)),
            NodeContent::FunctionDefinition(fd) => {
                Ok(Value::Func(Func::new_user_defined(ctx.clone(), fd.clone())))
            }
            NodeContent::List(list) => list
                .iter()
                .map(|x| x.resolve(ctx))
                .collect::<Result<Vec<Value>, ErrorWithLocation>>()
                .map(Into::into)
                .map(Value::List),
            NodeContent::HashMap(hm) => hm
                .iter()
                .map(|HmEntry { key, value }| {
                    Ok((
                        key.resolve(ctx)?
                            .as_value_string()
                            .map_err(|e| self.err(e.to_string()))?
                            .clone(),
                        value.resolve(ctx)?,
                    ))
                })
                .collect::<Result<HashMap<ValueString, Value>, ErrorWithLocation>>()
                .map(Rc::new)
                .map(Value::HashMap),
            NodeContent::FunctionCall {
                name: _,
                function,
                arguments,
            } => {
                let opt_args: Option<Vec<Value>> = arguments
                    .as_ref()
                    .map(|x| {
                        x.iter()
                            .map(|en| en.resolve(ctx))
                            .collect::<Result<Vec<Value>, ErrorWithLocation>>()
                    })
                    .map_or(Ok(None), |v| v.map(Some))?;
                match (&function.resolve(ctx)?, &opt_args) {
                    (Value::Func(func), Some(args)) => {
                        func.call(args.as_slice()) //.map_err(|e| self.add_location(e))
                    }
                    (_, Some(_)) => Err(self.err("value is not a function".to_string())),
                    (x, None) => Ok(x.clone()),
                }
            }
        }
    }

    fn err(&self, message: String) -> ErrorWithLocation {
        ErrorWithLocation {
            message,
            location: self.0.location.clone(),
        }
    }
}
