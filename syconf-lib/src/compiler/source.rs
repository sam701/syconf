use crate::compiler::Error;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct Source(Rc<SourceRef>);

#[derive(Debug)]
struct SourceRef {
    file: PathBuf,
    content: String,
}

impl Source {
    pub fn from_file(file_name: &Path) -> Result<Self, Error> {
        let mut content = String::new();
        let mut f = File::open(file_name)
            .map_err(|e| format!("Cannot open file '{}: {}", file_name.to_str().unwrap(), e))?;
        f.read_to_string(&mut content)
            .map_err(|e| format!("Cannot read file '{}': {}", file_name.to_str().unwrap(), e))?;

        Ok(Self(Rc::new(SourceRef {
            file: file_name.into(),
            content,
        })))
    }

    pub fn from_string(content: String) -> Self {
        Self(Rc::new(SourceRef {
            content,
            file: "<input_string>".into(),
        }))
    }

    pub fn file(&self) -> &PathBuf {
        &self.0.file
    }

    pub fn path(&self) -> &str {
        self.0.file.to_str().unwrap()
    }
}

impl Source {
    pub fn as_str(&self) -> &str {
        &self.0.content
    }
}

#[derive(Debug, Clone)]
pub struct Location {
    pub source: Source,
    pub position: usize,
}

impl std::fmt::Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let before = &self.source.as_str()[..self.position];
        let line_no = before.lines().count();
        write!(
            f,
            "{}:{}",
            &self.source.0.file.to_str().unwrap_or("somewhere"),
            line_no
        )
    }
}
