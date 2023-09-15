use crate::{
    code::{self, Span},
    enum_str,
    inst::{
        meta::*,
        op,
        util::{is_variant, Param},
        Emitter, ErrorMacro, Ops,
    },
};

#[derive(Debug)]
pub enum Error {}
type Result = std::result::Result<(), Error>;

def_directs! {
    global(Label()),
}

mod def {
    use super::*;

    pub fn global<E: Emitter>(e: &mut E, label: op::Label) -> Result {
        Ok(())
    }
}
