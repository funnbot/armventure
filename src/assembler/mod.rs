mod arg;
mod emit;
mod lex;
mod parse;

use crate::code::Source;
use bumpalo::Bump;

pub fn assemble(source: Source) {
    let ast_alloc = Bump::new();
    let mut parser = parse::Parser::new_in(&source, &ast_alloc);
    let emit_alloc = Bump::new();
    let mut emitter = emit::Emit::new_in(&source, &emit_alloc);
    while let Some(top) = parser.next() {
        emitter.process(&top);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{path::PathBuf, str::FromStr};

    #[test]
    fn it_assembles() {
        let text = "my_label: ADD x1, x2, x3\nB my_label\nB.EQ my_label\n";
        assemble(Source::new(
            PathBuf::from_str("some/path").unwrap(),
            String::from_str(text).unwrap(),
        ));
    }
}
