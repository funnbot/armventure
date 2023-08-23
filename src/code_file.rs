use std::num::NonZeroU32;
use std::path::PathBuf;
use std::{fmt, marker::PhantomData};
use thiserror::Error;
use typed_arena::Arena;

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to read file contents: {0}")]
    Read(#[from] std::io::Error),
}

type Result<T> = std::result::Result<T, Error>;

pub struct CodeFile {
    path: PathBuf,
}

impl CodeFile {
    pub fn new(path: PathBuf) -> Self {
        CodeFile { path }
    }

    pub fn load(self) -> Result<CodeSource> {
        let content = std::fs::read_to_string(self.path.as_path())?;
        Ok(CodeSource {
            path: self.path,
            content,
        })
    }
}

pub trait CodeFile2<'src> {
    type Diagnostic;
    fn path(&self) -> &std::path::Path;
    fn src(&self) -> &'src str;
    fn report(&mut self, span: CodeSpan<'src>, kind: Self::Diagnostic);
}

pub struct CodeSource {
    path: PathBuf,
    content: String,
}

impl CodeSource {
    pub fn content(&self) -> &str {
        self.content.as_str()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct CodeLoc {
    pub line: NonZeroU32,
    pub column: NonZeroU32,
}

pub struct Foo(Option<CodeLoc>);

pub struct CodeLoc2 {
    byte_index: usize,
    len: NonZeroU32,
    line: NonZeroU32,
    column: NonZeroU32,
}

pub struct Tokens {
    arena: Arena<CodeLoc2>,
}

impl CodeLoc2 {
    pub fn slice_of<'s>(&self, src: &'s str) -> &'s str {
        let end = self.byte_index + (self.len.get() as usize);
        &src[self.byte_index..end]
    }
}

impl CodeLoc {
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
}

impl Default for CodeLoc {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy)]
pub struct CodeSpan<'a> {
    pub loc: CodeLoc,
    pub src: &'a str,
    pub byte_index: usize,
}

impl<'src> CodeSpan<'src> {
    pub fn len(&self) -> usize {
        self.src.len()
    }
    pub fn is_empty(&self) -> bool {
        self.src.is_empty()
    }
}

/// # Safety
/// fst and snd must be from the same allocated objects
unsafe fn join_adj_slices<'a, T>(fst: &'a [T], snd: &'a [T]) -> &'a [T] {
    let fst_end = fst.as_ptr().add(fst.len());
    let snd_start = snd.as_ptr();
    assert_eq!(fst_end, snd_start, "slices must be contiguous");
    // The assertion above ensures `fst` and `snd` are contiguous, but they might
    // still be contained within _different allocated objects_, in which case
    // creating this slice is undefined behavior.
    std::slice::from_raw_parts(fst.as_ptr(), fst.len() + snd.len())
}

/// # Safety
/// fst and snd must be from the same allocated objects
unsafe fn join_adj_strs<'a>(fst: &'a str, snd: &'a str) -> &'a str {
    std::str::from_utf8_unchecked(join_adj_slices(
        fst.as_bytes(),
        snd.as_bytes(),
    ))
}

impl<'src> CodeSpan<'src> {
    /// # Safety
    /// self and snd must be from the same allocated objects
    pub unsafe fn concat_adj(&'src mut self, snd: &'src Self) {
        self.src = join_adj_strs(self.src, snd.src);
    }
}

impl fmt::Debug for CodeLoc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

impl fmt::Debug for CodeSpan<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Span {{{:?} \"{}\"}}", self.loc, self.src)
    }
}
