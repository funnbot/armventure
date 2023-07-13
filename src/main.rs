use std::{fs, io, path::Path};

use armventure::assembler;
use armventure::scan;

// pub mod assembler;
// pub mod multipeekable;
// pub mod scanner;
// pub mod source_file;

fn read_file(path: &Path) -> io::Result<Vec<u8>> {
    return fs::read(path);
}

fn main() {
    println!("Hello, world!");
}
