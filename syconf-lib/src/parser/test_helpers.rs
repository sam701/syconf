use crate::parser::Span;

pub fn span(content: &str) -> Span {
    Span::new_extra(content, "<test_string>".into())
}
