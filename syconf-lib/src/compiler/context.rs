use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::node::CodeNode;

#[derive(Clone, Debug)]
pub struct Context(Rc<RefCell<ContextRef>>);

#[derive(Debug)]
struct ContextRef {
    bindings: HashMap<String, CodeNode>,
    parent: Option<Context>,
}

impl Context {
    pub fn empty() -> Self {
        Self(Rc::new(RefCell::new(ContextRef {
            bindings: HashMap::new(),
            parent: None,
        })))
    }

    pub fn get_value(&self, val: &str) -> Option<CodeNode> {
        let x = self.0.borrow();
        x.bindings
            .get(val)
            .map(Clone::clone)
            .or_else(|| x.parent.as_ref().and_then(|p| p.get_value(val)))
    }

    pub fn new_child(&self) -> Self {
        Self(Rc::new(RefCell::new(ContextRef {
            bindings: HashMap::new(),
            parent: Some(self.clone()),
        })))
    }

    pub fn bind(&self, key: String, value: CodeNode) {
        self.0.borrow_mut().bindings.insert(key, value);
    }
}
