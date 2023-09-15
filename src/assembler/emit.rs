use super::{
    arg,
    parse::ast::{self, Top},
};
use crate::{
    bitstack::{push_bits_offset_u32, BitStackU32},
    code,
    enum_str::EnumStr,
    inst::{
        self, apply_label_fixup, label, operand::Ops, Emitter, EncInstr, EncInstrSet, Error,
        ErrorMacro, Fixup, Mnemonic,
    },
    sparsebin::{Aligned, SparseBin},
};
use bit::{BitCt, Int, IntN};
use bumpalo::Bump;
use rustc_hash::FxHashMap as HashMap;
use std::{cell::Cell, str::FromStr};

struct LabelResolver<E: Emitter> {
    intern: label::Intern,
    addr_map: HashMap<label::Key, u64>,
    fixups: Vec<Fixup<E, label::Key, u64>>,
}

impl LabelResolver<Emit<'_, '_>> {
    pub fn new() -> Self {
        Self {
            intern: label::Intern::new(),
            addr_map: HashMap::default(),
            fixups: Vec::new(),
        }
    }
}

pub struct Emit<'bump, 'src> {
    bin: SparseBin,
    pc: u64,
    bit_stack: BitStackU32,
    labels: LabelResolver<Self>,
    bump: &'bump Bump,
    ops_vec: Cell<Vec<Ops>>,
    src: &'src code::Source,
}

impl Emitter for Emit<'_, '_> {
    fn pc(&self) -> u64 {
        self.pc
    }

    fn bit_idx(&self) -> u8 {
        self.bit_stack.len()
    }

    fn set_pc(&mut self, value: u64) {
        self.pc = value
    }

    fn push(&mut self, value: IntN) {
        self.bit_stack.push(value.0, value.1);
    }

    fn push_n<const N: BitCt>(&mut self, value: Int<N>) {
        self.bit_stack.push(value.0, N as u8);
    }

    fn insert(&mut self, value: IntN, offset: u8) {
        let addr = Aligned::new(self.pc as usize).unwrap();
        let current = self.bin.get_u32(addr);
        let result = push_bits_offset_u32(current, value.0, value.1, offset);
        self.bin.write_u32(addr, result);
    }

    fn begin_instr(&mut self) {
        self.bit_stack = BitStackU32::new();
    }

    fn end_instr(&mut self) {
        assert!(self.bit_stack.all_bits_written());
        let value = self.bit_stack.value();
        self.bin
            .write_u32(Aligned::new(self.pc as usize).unwrap(), value);
        self.pc += 4;
    }

    fn resolve_label(&mut self, key: label::Key) -> Option<u64> {
        self.labels.addr_map.get(&key).copied()
    }

    fn push_label_fixup(&mut self, fixup: Fixup<Self, label::Key, u64>) {
        self.labels.fixups.push(fixup);
    }
}

pub struct NoDrop<T>(pub T);
impl<T> std::ops::Drop for NoDrop<T> {
    fn drop(&mut self) {
        panic!("NoDrop dropped");
    }
}
impl<T> std::ops::Deref for NoDrop<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T> std::ops::DerefMut for NoDrop<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'bump, 'src> Emit<'bump, 'src> {
    pub fn new_in(src: &'src code::Source, bump: &'bump Bump) -> Self {
        Emit {
            bin: SparseBin::new(),
            pc: 0,
            bit_stack: BitStackU32::new(),
            labels: LabelResolver::new(),
            ops_vec: Cell::new(Vec::new()),
            bump,
            src,
        }
    }

    fn take_ops_vec(&self) -> NoDrop<Vec<Ops>> {
        NoDrop(self.ops_vec.take())
    }
    fn set_ops_vec(&self, mut ops_vec: NoDrop<Vec<Ops>>) {
        self.ops_vec.set(std::mem::take(&mut ops_vec));
        std::mem::forget(ops_vec);
    }

    pub fn process<'ast>(&mut self, top: &'ast ast::Top<'ast>) {
        match top {
            Top::Instruction {
                args,
                mnem: mnem_span,
            } => {
                let Some(mnem) = Mnemonic::from_str_lower_or_upper(self.src.span(*mnem_span))
                else {
                    self.src.report(*mnem_span, "unknown mnemonic");
                    return;
                };
                let narrow = inst::narrow_variant(mnem);

                let arg_len = args.map_or(0, |a| a.len());
                let mut ops_vec = self.take_ops_vec();

                ops_vec.clear();
                ops_vec.reserve_exact(arg_len);

                let mut arg_parser = arg::ArgParser::new(self.src, &mut self.labels.intern, narrow);
                if let Some(args) = args {
                    arg_parser.parse_args(args, &mut ops_vec);
                }

                if let Ok(variant) = arg_parser.finish().map_err(|e| {
                    self.src
                        .report(*mnem_span, format_args!("TODO: NarrowError {:?}", e))
                }) {
                    if let Err(e) = inst::get_variant_and_emit(mnem, variant, ops_vec.iter(), self)
                    {
                        self.handle_error(e, *mnem_span);
                    }
                }

                self.set_ops_vec(ops_vec);
            }
            Top::Directive {
                name: name_span,
                args,
            } => {
                let Some(name) =
                    inst::dir::Name::from_str_lower_or_upper(self.src.span(*name_span))
                else {
                    self.src.report(*name_span, "unknown directive");
                    return;
                };

                let arg_len = args.map(|a| a.len()).unwrap_or(0);
                let mut ops_vec = self.take_ops_vec();

                ops_vec.clear();
                ops_vec.reserve_exact(arg_len);

                let narrow = inst::dir::narrow_variant(name);
                let mut arg_parser = arg::ArgParser::new(self.src, &mut self.labels.intern, narrow);
                args.map(|args| arg_parser.parse_args(args, &mut ops_vec));
                inst::dir::select_and_run(self, ops_vec.iter()).unwrap_or_else(|e| {
                    self.src
                        .report(*name_span, format_args!("TODO: DirectiveError {:?}", e));
                });
                self.set_ops_vec(ops_vec);
            }
            Top::Label(span) => {
                let str = self.src.span(*span);
                let key = self.labels.intern.get_or_intern(str);
                if self.labels.addr_map.insert(key, self.pc()).is_some() {
                    //self.src.report(*span, "label already defined");
                    todo!("error if label already exists");
                }
            }
            Top::Error => {
                // error already reported
            }
        }
    }

    fn handle_error(&self, ErrorMacro(e, s): ErrorMacro, span: code::Span) {
        todo!("src.report error here, with span from process")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{assembler::parse::Parser, code::Source};
    use lasso::Rodeo;
    use std::{hash::BuildHasherDefault, path::PathBuf};

    #[test]
    fn try_it() {
        let src = Source::new(PathBuf::new(), String::new());
        let emit_alloc = Bump::new();
        let mut rodeo = label::Intern::new();
        let label = rodeo.get_or_intern_static("my_label");
        let args = &[Ops::Label(label)];
        let args2 = &[Ops::Label(label)];
        let mnemonic = Mnemonic::from_str("B").unwrap();
        // let instr = parse_variant(mnemonic, args.iter()).unwrap();
        // let instr2 = parse_variant(Mnemonic::from_str("B").unwrap(), args2.iter()).unwrap();
        let mut e = Emit::new_in(&src, &emit_alloc);

        let addr = 0;
        e.set_pc(addr);

        // emit_instr(instr.unwrap(), &mut e).unwrap();
        // emit_instr(instr2.unwrap(), &mut e).unwrap();
        e.labels.addr_map.insert(label, 0xFF0);

        println!(
            "0b{:032b}",
            e.bin.get_u32(Aligned::new(addr as usize).unwrap())
        );
        println!("{:?}", e.labels.fixups[0]);
        let fixup = e.labels.fixups[0].clone();
        apply_label_fixup(&mut e, fixup).unwrap();
        let fixup2 = e.labels.fixups[1].clone();
        apply_label_fixup(&mut e, fixup2).unwrap();
        println!(
            "0b{:032b}",
            e.bin.get_u32(Aligned::new(addr as usize).unwrap())
        );
    }
}
