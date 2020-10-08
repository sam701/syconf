use std::rc::Rc;
use std::path::{PathBuf, Path};
use crate::compiler::Error;
use std::fs::File;
use std::io::Read;

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
        let mut f = File::open(file_name)?;
        f.read_to_string(&mut content)?;

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
}

impl Source {
    pub fn as_str(&self) -> &str {
        &self.0.content
    }
}

#[derive(Debug)]
pub struct Location {
    pub source: Source,
    pub position: usize,
}
