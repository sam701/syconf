use crate::parser::Span;
use std::rc::Rc;

pub fn span(content: &str) -> Span {
    Span::new_extra(content, Rc::new("<test_string>".to_owned()))
}
