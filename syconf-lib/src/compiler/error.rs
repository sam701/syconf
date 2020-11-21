use std::cmp::min;

use nom::error::ErrorKind;
use nom::{Err, InputLength};

use crate::compiler::value::TypeMismatch;
use crate::parser::Span;

use std::sync::Arc;

pub type Error = ErrorWithLocation;

#[derive(thiserror::Error, Debug)]
pub struct ErrorWithLocation {
    pub location: Option<Location>,
    pub message: String,
}

impl std::fmt::Display for ErrorWithLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = self
            .location
            .as_ref()
            .map(|l| format!("{}", l))
            .unwrap_or_else(|| "somewhere".to_string());

        write!(f, "{}: {}", &s, self.message)
    }
}

impl From<anyhow::Error> for ErrorWithLocation {
    fn from(e: anyhow::Error) -> Self {
        ErrorWithLocation {
            location: None,
            message: e.to_string(),
        }
    }
}

impl From<String> for ErrorWithLocation {
    fn from(e: String) -> Self {
        ErrorWithLocation {
            location: None,
            message: e,
        }
    }
}

impl From<&str> for ErrorWithLocation {
    fn from(e: &str) -> Self {
        ErrorWithLocation {
            location: None,
            message: e.to_string(),
        }
    }
}

impl From<TypeMismatch> for ErrorWithLocation {
    fn from(e: TypeMismatch) -> Self {
        ErrorWithLocation {
            location: None,
            message: e.to_string(),
        }
    }
}

impl<'a> From<&Span<'a>> for ErrorWithLocation {
    fn from(loc: &Span<'a>) -> Self {
        ErrorWithLocation {
            location: Some(loc.into()),
            message: format!(
                "Cannot parse: '{}'",
                &loc.fragment()[..min(20, loc.input_len())]
            ),
        }
    }
}

impl<'a> From<Err<(Span<'a>, ErrorKind)>> for ErrorWithLocation {
    fn from(e: Err<(Span<'a>, ErrorKind)>) -> Self {
        match e {
            Err::Incomplete(_) => ErrorWithLocation {
                location: None,
                message: "Incomplete input".to_owned(),
            },
            Err::Error((loc, _)) => (&loc).into(),
            Err::Failure((loc, _)) => (&loc).into(),
        }
    }
}

#[macro_export]
macro_rules! check {
    ($cond:expr, $msg:literal $(,)?) => {
        if !$cond {
            return Err($crate::compiler::error::ErrorWithLocation{
                location: None,
                message: $msg.to_string(),
            });
        }
    };
    ($cond:expr, $fmt:expr, $($arg:tt)*) => {
        if !$cond {
            return Err($crate::compiler::error::ErrorWithLocation{
                location: None,
                message: format!($fmt, $($arg)*),
            });
        }
    };
}

#[derive(Debug, Clone)]
pub struct Location {
    pub source: Arc<str>,
    pub line: usize,
    pub column: usize,
    pub offset: usize,
}

impl std::fmt::Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}", &self.source, &self.line, self.column,)
    }
}

impl<'a> From<&Span<'a>> for Location {
    fn from(loc: &Span<'a>) -> Self {
        Self {
            source: loc.extra.clone(),
            line: loc.location_line() as usize,
            column: loc.get_column(),
            offset: loc.location_offset(),
        }
    }
}

#[test]
fn error_location() {
    let result = crate::parse_string(" abc");
    let err = result.err().unwrap();
    let loc = err.location;
    assert_eq!(loc.unwrap().line, 1);
}
