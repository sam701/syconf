use std::fs::read_to_string;
use std::path::Path;
use std::rc::Rc;

use crate::compiler::context::Context;
use crate::compiler::functions::FunctionSig;
use crate::compiler::node::{CodeNode, FunctionDefinition, HmEntry, NodeContent};
use crate::compiler::value::{Func, Value};
use crate::compiler::{methods, operators, Error, ErrorWithLocation};
use crate::parser::string::ConfigString;
use crate::parser::*;
use crate::parser::{Expr, ExprWithLocation};

pub struct Compiler;

impl Compiler {
    pub fn compile(&self, ctx: &Context, expr: &ExprWithLocation) -> Result<CodeNode, Error> {
        let cell = match &expr.inner {
            Expr::Value(val) => self.config_value(ctx, val)?,
            Expr::Block(block) => return self.block(ctx, block),
            Expr::Identifier(id) => self.identifier(ctx, id, &expr.location)?,
            Expr::FuncDefinition(fd) => self.func_definition(ctx, fd)?,
            Expr::Math(op) => self.math_op(ctx, op)?,
            Expr::Comparison(cmp) => self.comparison(ctx, cmp)?,
            Expr::Conditional(cond) => self.conditional(ctx, cond)?,
            Expr::Logical(logical) => self.logical(ctx, logical)?,
            Expr::Suffix(suffix) => self.suffix_operator(ctx, suffix)?,
            Expr::Import(path) => return self.import(path, &expr.location),
        };
        Ok(CodeNode::new(cell, Some((&expr.location).into())))
    }

    fn suffix_operator(&self, ctx: &Context, suffix: &SuffixExpr) -> Result<NodeContent, Error> {
        let base = self.compile(ctx, &suffix.base)?;
        debug!(?suffix, "suffix_op");
        let args = match &suffix.operator {
            SuffixOperator::FunctionApplication(args) => {
                return Ok(NodeContent::FunctionCall {
                    name: ".apply".to_string(),
                    function: base,
                    arguments: Some(args.iter().map(|x| self.compile(ctx, x)).collect::<Result<
                        Vec<CodeNode>,
                        Error,
                    >>(
                    )?),
                });
            }
            SuffixOperator::DotField(id) => vec![
                base,
                CodeNode::new(NodeContent::Resolved(Value::String((*id).into())), None),
            ],
            SuffixOperator::Index(ix) => vec![base, self.compile(ctx, ix)?],
        };
        Ok(NodeContent::FunctionCall {
            name: ".get".to_string(),
            function: builtin_func_node(&methods::index),
            arguments: Some(args),
        })
    }

    fn logical(&self, ctx: &Context, logical: &Logical) -> Result<NodeContent, Error> {
        let (func, name, args): (&'static FunctionSig, &str, Vec<CodeNode>) = match logical {
            Logical::And(expr1, expr2) => (
                &operators::and,
                "and",
                vec![self.compile(ctx, &expr1)?, self.compile(ctx, &expr2)?],
            ),
            Logical::Or(expr1, expr2) => (
                &operators::or,
                "or",
                vec![self.compile(ctx, &expr1)?, self.compile(ctx, &expr2)?],
            ),
            Logical::Not(expr1) => (&operators::not, "not", vec![self.compile(ctx, &expr1)?]),
        };
        Ok(NodeContent::FunctionCall {
            name: name.to_string(),
            function: builtin_func_node(func),
            arguments: Some(args),
        })
    }

    fn conditional(&self, ctx: &Context, cond: &Conditional) -> Result<NodeContent, Error> {
        let args = vec![
            self.compile(ctx, &cond.condition)?,
            self.compile(ctx, &cond.then_branch)?,
            self.compile(ctx, &cond.else_branch)?,
        ];
        Ok(NodeContent::FunctionCall {
            name: "if".to_string(),
            function: CodeNode::new(
                NodeContent::Resolved(Value::Func(Func::new_builtin(&operators::conditional))),
                None,
            ),
            arguments: Some(args),
        })
    }

    fn comparison(&self, ctx: &Context, cmp: &Comparison) -> Result<NodeContent, Error> {
        let args = vec![
            self.compile(ctx, &cmp.expr1)?,
            self.compile(ctx, &cmp.expr2)?,
        ];
        Ok(NodeContent::FunctionCall {
            name: format!("{:?}", cmp.operator),
            function: CodeNode::new(
                NodeContent::Resolved(Value::Func(Func::new_builtin(operators::comparison(
                    &cmp.operator,
                )))),
                None,
            ),
            arguments: Some(args),
        })
    }

    fn math_op(&self, ctx: &Context, op: &MathOperation) -> Result<NodeContent, Error> {
        let args = vec![self.compile(ctx, &op.expr1)?, self.compile(ctx, &op.expr2)?];
        Ok(NodeContent::FunctionCall {
            name: format!("{:?}", op.op),
            function: CodeNode::new(
                NodeContent::Resolved(Value::Func(Func::new_builtin(operators::math(&op.op)))),
                None,
            ),
            arguments: Some(args),
        })
    }

    fn config_value(&self, ctx: &Context, val: &ConfigValue) -> Result<NodeContent, Error> {
        match val {
            ConfigValue::Bool(x) => Ok(NodeContent::Resolved(Value::Bool(*x))),
            ConfigValue::Int(v) => Ok(NodeContent::Resolved(Value::Int(*v))),
            ConfigValue::String(s) => self.string(ctx, s),
            ConfigValue::HashMap(hm) => hm
                .iter()
                .map(|HashMapEntry { key, value }| {
                    Ok(HmEntry {
                        key: self.compile(ctx, key)?,
                        value: self.compile(ctx, value)?,
                    })
                })
                .collect::<Result<Vec<HmEntry>, Error>>()
                .map(NodeContent::HashMap),
            ConfigValue::List(list) => list
                .iter()
                .map(|x| self.compile(ctx, x))
                .collect::<Result<Vec<CodeNode>, Error>>()
                .map(NodeContent::List),
        }
    }

    fn string(&self, ctx: &Context, cs: &[ConfigString]) -> Result<NodeContent, Error> {
        let kids = cs
            .iter()
            .map(|x| match x {
                ConfigString::Raw(s) => Ok(CodeNode::new(
                    NodeContent::Resolved(Value::String((*s).into())),
                    None,
                )),
                ConfigString::Interpolated(a) => self.compile(ctx, a),
            })
            .collect::<Result<Vec<CodeNode>, Error>>()?;
        Ok(NodeContent::FunctionCall {
            function: builtin_func_node(&super::functions::concat_strings),
            arguments: Some(kids),
            name: "concat".to_string(),
        })
    }

    fn block(&self, ctx: &Context, block: &BlockExpr) -> Result<CodeNode, Error> {
        let ns = ctx.new_child();
        debug!(?block.local_assignments, "blocqk");
        for Assignment(id, ex) in &block.local_assignments {
            debug!(?id, ?ex, "assignment1");
            let node = self.compile(&ns, &ex)?;
            debug!(?id, ?node, "assignment2: binding {}", id);
            ns.bind(id.to_string(), node);
        }
        self.compile(&ns, &block.expression)
    }

    fn identifier(&self, ctx: &Context, id: &str, loc: &Span) -> Result<NodeContent, Error> {
        let func_node = ctx
            .get_value(id)
            .or_else(|| super::functions::lookup(id).map(|func| builtin_func_node(func)))
            .ok_or_else(|| ErrorWithLocation {
                location: Some(loc.into()),
                message: format!("Variable '{}' is not defined", id),
            })?;
        Ok(NodeContent::FunctionCall {
            name: id.to_string(),
            function: func_node,
            arguments: None,
        })
    }

    fn func_definition(&self, ctx: &Context, fd: &FuncDefinition) -> Result<NodeContent, Error> {
        debug!(?fd.arguments, "function definition");
        let ns = ctx.new_child();
        for arg in &fd.arguments {
            ns.bind(
                arg.to_string(),
                CodeNode::new(NodeContent::FunctionInputArgument(arg.to_string()), None),
            );
        }
        let val = self.compile(&ns, &fd.expression)?;
        let string_args: Vec<String> = fd.arguments.iter().map(|x| x.to_string()).collect();
        Ok(NodeContent::FunctionDefinition(Rc::new(
            FunctionDefinition {
                node: val,
                argument_names: Some(string_args),
            },
        )))
    }

    fn import(&self, file_name: &str, location: &Span) -> Result<CodeNode, Error> {
        let final_file_name = Path::new(location.extra.as_str())
            .parent()
            .unwrap()
            .join(file_name)
            .canonicalize()
            .unwrap();
        let content = read_to_string(&final_file_name).map_err(|e| ErrorWithLocation {
            location: Some(location.into()),
            message: format!(
                "Cannot read file '{}': {}",
                final_file_name.to_str().unwrap(),
                &e
            ),
        })?;
        let (_, expr) = parse_unit(Span::new_extra(
            &content,
            Rc::new(final_file_name.to_str().unwrap().to_owned()),
        ))?;
        Compiler.compile(&Context::empty(), &expr)
    }
}

fn builtin_func_node(func: &'static (dyn Fn(&[Value]) -> Result<Value, Error>)) -> CodeNode {
    CodeNode::new(
        NodeContent::Resolved(Value::Func(Func::new_builtin(func))),
        None,
    )
}
