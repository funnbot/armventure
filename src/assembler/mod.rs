pub mod lex;
pub mod parse;
pub mod emit;

use crate::code_file::CodeSource;
use bumpalo::Bump;

pub fn assemble(file: &CodeSource) {
    let ast_alloc = Bump::new();
    let parser = parse::Parser::new_in(
        lex::Scanner::new(file.content()),
        &ast_alloc,
    );
    let emit_alloc = Bump::new();

}
