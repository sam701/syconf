use std::fs::read_to_string;
use std::path::Path;
use std::sync::Arc;

use crate::parser::string::ConfigString;
use crate::parser::*;
use crate::parser::{Expr, ExprWithLocation};
use crate::resolver::context::Context;
use crate::resolver::node::{CodeNode, FunctionDefinition, HmEntry, NodeContent};
use crate::resolver::value::{Func, FunctionSig, Value};
use crate::resolver::{methods, operators, Error, ErrorWithLocation};

pub struct NodeTreeBuilder;

impl NodeTreeBuilder {
    pub fn build_tree(&self, ctx: &Context, expr: &ExprWithLocation) -> Result<CodeNode, Error> {
        let cell = match &expr.inner {
            Expr::Value(val) => self.config_value(ctx, val)?,
            Expr::Block(block) => return self.block(ctx, block),
            Expr::Identifier(id) => self.identifier(ctx, id, &expr.location)?,
            Expr::FuncDefinition(fd) => self.func_definition(ctx, fd)?,
            Expr::BinaryOperator(op) => self.math_op(ctx, op)?,
            Expr::Comparison(cmp) => self.comparison(ctx, cmp)?,
            Expr::Conditional(cond) => self.conditional(ctx, cond)?,
            Expr::Logical(logical) => self.logical(ctx, logical)?,
            Expr::Suffix(suffix) => self.suffix_operator(ctx, suffix)?,
            Expr::Import(path) => return self.import(path, ctx, &expr.location),
        };
        Ok(CodeNode::new(cell, Some((&expr.location).into())))
    }

    fn suffix_operator(&self, ctx: &Context, suffix: &SuffixExpr) -> Result<NodeContent, Error> {
        let base = self.build_tree(ctx, &suffix.base)?;
        debug!(?suffix, "suffix_op");
        let args = match &suffix.operator {
            SuffixOperator::FunctionApplication(args) => {
                return Ok(NodeContent::FunctionCall {
                    name: ".apply".to_string(),
                    function: base,
                    arguments: Some(
                        args.iter()
                            .map(|x| self.build_tree(ctx, x))
                            .collect::<Result<Vec<CodeNode>, Error>>()?,
                    ),
                });
            }
            SuffixOperator::DotField(id) => vec![
                base,
                CodeNode::new(NodeContent::Resolved(Value::String((*id).into())), None),
            ],
            SuffixOperator::Index(ix) => vec![base, self.build_tree(ctx, ix)?],
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
                vec![self.build_tree(ctx, expr1)?, self.build_tree(ctx, expr2)?],
            ),
            Logical::Or(expr1, expr2) => (
                &operators::or,
                "or",
                vec![self.build_tree(ctx, expr1)?, self.build_tree(ctx, expr2)?],
            ),
            Logical::Not(expr1) => (&operators::not, "not", vec![self.build_tree(ctx, expr1)?]),
        };
        Ok(NodeContent::FunctionCall {
            name: name.to_string(),
            function: builtin_func_node(func),
            arguments: Some(args),
        })
    }

    fn conditional(&self, ctx: &Context, cond: &Conditional) -> Result<NodeContent, Error> {
        Ok(NodeContent::Conditional {
            condition: self.build_tree(ctx, &cond.condition)?,
            then_branch: self.build_tree(ctx, &cond.then_branch)?,
            else_branch: self.build_tree(ctx, &cond.else_branch)?,
        })
    }

    fn comparison(&self, ctx: &Context, cmp: &Comparison) -> Result<NodeContent, Error> {
        let args = vec![
            self.build_tree(ctx, &cmp.expr1)?,
            self.build_tree(ctx, &cmp.expr2)?,
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

    fn math_op(&self, ctx: &Context, op: &BinaryOperatorExpr) -> Result<NodeContent, Error> {
        let args = vec![
            self.build_tree(ctx, &op.expr1)?,
            self.build_tree(ctx, &op.expr2)?,
        ];
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
            ConfigValue::Number(v) => Ok(NodeContent::Resolved(Value::Number(v.clone()))),
            ConfigValue::String(s) => self.string(ctx, s),
            ConfigValue::HashMap(hm) => hm
                .iter()
                .map(|HashMapEntry { key, value }| {
                    Ok(HmEntry {
                        key: self.build_tree(ctx, key)?,
                        value: self.build_tree(ctx, value)?,
                    })
                })
                .collect::<Result<Vec<HmEntry>, Error>>()
                .map(NodeContent::HashMap),
            ConfigValue::List(list) => list
                .iter()
                .map(|x| self.build_tree(ctx, x))
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
                ConfigString::Interpolated(a) => self.build_tree(ctx, a),
            })
            .collect::<Result<Vec<CodeNode>, Error>>()?;
        Ok(NodeContent::FunctionCall {
            function: builtin_func_node(&super::functions::concat_strings),
            arguments: Some(kids),
            name: "string_concat".to_string(),
        })
    }

    fn block(&self, ctx: &Context, block: &BlockExpr) -> Result<CodeNode, Error> {
        let ns = ctx.new_child();
        debug!(?block.local_assignments, "block");
        for Assignment(id, ex) in &block.local_assignments {
            debug!(?id, ?ex, "assignment1");
            let node = self.build_tree(&ns, ex)?;
            debug!(?id, ?node, "assignment2: binding {}", id);
            ns.bind(id.to_string(), node);
        }
        self.build_tree(&ns, &block.expression)
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
        let val = self.build_tree(&ns, &fd.expression)?;
        let string_args: Vec<String> = fd.arguments.iter().map(|x| x.to_string()).collect();
        Ok(NodeContent::FunctionDefinition(Arc::new(
            FunctionDefinition {
                node: val,
                argument_names: Some(string_args),
            },
        )))
    }

    fn import(&self, file_name: &str, ctx: &Context, location: &Span) -> Result<CodeNode, Error> {
        let final_file_name = Path::new(&*location.extra)
            .parent()
            .unwrap()
            .join(file_name)
            .canonicalize()
            .unwrap();
        let file_name_str = final_file_name.to_str().expect("absolute path").to_string();
        if let Some(node) = ctx.get_value(&file_name_str) {
            debug!(%file_name_str, "Found in cache");
            return Ok(node);
        }
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
            final_file_name.to_str().unwrap().into(),
        ))?;
        let node = NodeTreeBuilder.build_tree(&Context::empty(), &expr)?;
        ctx.bind(file_name_str, node.clone());
        Ok(node)
    }
}

fn builtin_func_node(func: &'static FunctionSig) -> CodeNode {
    CodeNode::new(
        NodeContent::Resolved(Value::Func(Func::new_builtin(func))),
        None,
    )
}
