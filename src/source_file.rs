use std::path::PathBuf;

pub enum Error {
    Read(std::io::Error),
}

type Result<T> = std::result::Result<T, Error>;

pub struct SourceFile {
    path: PathBuf,
    content: Option<String>,
}

impl SourceFile {
    pub fn new(path: PathBuf) -> Self {
        SourceFile {
            path,
            content: None,
        }
    }

    pub fn content(&mut self) -> Result<&str> {
        if self.content.is_none() {
            self.content = match std::fs::read_to_string(self.path.as_path()) {
                Ok(content) => Some(content),
                Err(e) => return Err(Error::Read(e)),
            }
        }
        
        return Ok(self.content.as_ref().unwrap().as_str());
    }
}
