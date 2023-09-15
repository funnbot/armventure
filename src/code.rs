use std::{
    cell::Cell,
    fmt,
    marker::PhantomData,
    num::{NonZeroU32, NonZeroU8},
    ops::Range,
    path::PathBuf,
};
use thiserror::Error;
use typed_arena::Arena;

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to read file contents: {0}")]
    Read(#[from] std::io::Error),
}

type Result<T> = std::result::Result<T, Error>;

pub struct File {
    path: PathBuf,
}

impl File {
    pub fn new(path: PathBuf) -> Self {
        File { path }
    }

    pub fn load(self) -> Result<Source> {
        let content = std::fs::read_to_string(self.path.as_path())?;
        Ok(Source {
            path: self.path,
            content,
            diags: Cell::new(Vec::new()),
        })
    }
}

pub struct Source {
    path: PathBuf,
    content: String,
    diags: Cell<Vec<String>>,
}

impl Source {
    pub fn new(path: PathBuf, content: String) -> Self {
        Source {
            path,
            content,
            diags: Cell::new(Vec::new()),
        }
    }
    pub fn str(&self) -> &str {
        self.content.as_str()
    }
    pub fn create_span(&self, loc: Loc, start: usize, end: usize) -> Span {
        let _ = self.content[start..end];
        Span { loc, start, end }
    }
    pub fn span(&self, span: Span) -> &str {
        &self.content[span.start..span.end]
    }
    pub fn report<S: ToString>(&self, span: Span, msg: S) {
        let mut vec = self.diags.take();
        vec.push(msg.to_string());

        eprintln!(
            "{}:{}:{}: {}",
            self.path.display(),
            span.loc.line(),
            span.loc.column(),
            vec.last().unwrap(),
        );

        self.diags.set(vec);
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Loc {
    line: NonZeroU32,
    column: NonZeroU32,
}

impl Loc {
    #[must_use]
    #[allow(clippy::missing_panics_doc, reason = "const")]
    pub fn new() -> Self {
        Self {
            line: NonZeroU32::new(1).unwrap(),
            column: NonZeroU32::new(1).unwrap(),
        }
    }
    /// # Panics
    /// out of range
    pub fn inc_line(&mut self) {
        self.line = self.line.checked_add(1).unwrap();
        self.column = NonZeroU32::new(1).unwrap();
    }
    /// # Panics
    /// out of range
    pub fn inc_column(&mut self) {
        self.column = self.column.checked_add(1).unwrap();
    }

    pub fn line(&self) -> u32 {
        self.line.get()
    }
    pub fn column(&self) -> u32 {
        self.column.get()
    }

    pub fn add(&mut self, line: u32, column: u32) {
        self.line.checked_add(line).unwrap();
        self.column.checked_add(column).unwrap();
    }
}

impl Default for Loc {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy)]
pub struct Span {
    loc: Loc,
    //pub src: &'a str,
    start: usize,
    end: usize,
}

impl Span {
    pub fn len(&self) -> usize {
        self.end - self.start
    }
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
    pub fn loc(&self) -> Loc {
        self.loc
    }
    pub fn group(start: Span, end: Span) -> Span {
        Span {
            loc: start.loc,
            start: start.start,
            end: end.end,
        }
    }
}

impl fmt::Debug for Loc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

impl fmt::Debug for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Span {{{:?}}}", self.loc)
    }
}
