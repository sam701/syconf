use std::collections::HashMap;

use super::node::CodeNode;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
pub struct Context(Arc<Mutex<ContextRef>>);

#[derive(Debug)]
struct ContextRef {
    bindings: HashMap<String, CodeNode>,
    parent: Option<Context>,
}

impl Context {
    pub fn empty() -> Self {
        Self(Arc::new(Mutex::new(ContextRef {
            bindings: HashMap::new(),
            parent: None,
        })))
    }

    pub fn get_value(&self, val: &str) -> Option<CodeNode> {
        let x = self.0.lock().expect("cannot lock");
        x.bindings
            .get(val)
            .cloned()
            .or_else(|| x.parent.as_ref().and_then(|p| p.get_value(val)))
    }

    pub fn new_child(&self) -> Self {
        Self(Arc::new(Mutex::new(ContextRef {
            bindings: HashMap::new(),
            parent: Some(self.clone()),
        })))
    }

    pub fn bind(&self, key: String, value: CodeNode) {
        self.0
            .lock()
            .expect("cannot lock")
            .bindings
            .insert(key, value);
    }
}
