use crate::code_file::CodeSource;
use crate::scan::{CodeLoc, CodeSpan};

struct Diag<'src, Kind> {
    span: CodeSpan<'src>,
    kind: Kind,
}

pub struct Reporter<'src, Kind> {
    diags: Vec<Diag<'src, Kind>>,
}

impl<'src, Kind> Reporter<'src, Kind> {
    pub fn report(&mut self, span: CodeSpan<'src>, kind: Kind) {
        self.diags.push(Diag { span, kind })
    }
}
