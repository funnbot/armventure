use std::str::FromStr;

use super::parse::ast;
use crate::{
    bitwriter::{bit_write_u32, BitWriter, BitWriterU32},
    inst::apply_label_fixup,
    inst::parse::*,
    sparsebin::{Aligned, SparseBin},
};
use lasso::Spur;
use rustc_hash::FxHashMap as HashMap;

pub type Interner = lasso::Rodeo<Spur, core::hash::BuildHasherDefault<rustc_hash::FxHasher>>;

struct LabelResolver {
    addr_map: HashMap<Spur, u64>,
    fixups: Vec<Fixup<Emit, Spur, u64>>,
}

impl LabelResolver {
    pub fn new() -> Self {
        Self {
            addr_map: HashMap::default(),
            fixups: Vec::new(),
        }
    }
}

struct Emit {
    bin: SparseBin,
    pc: u64,
    writer: BitWriterU32,
    labels: LabelResolver,
}

impl Emit {
    pub fn new() -> Self {
        Emit {
            bin: SparseBin::new(),
            pc: 0,
            writer: BitWriterU32::new(),
            labels: LabelResolver::new(),
        }
    }
}

impl Emitter for Emit {
    fn pc(&self) -> u64 {
        self.pc
    }

    fn bit_idx(&self) -> u8 {
        self.writer.bit_idx()
    }

    fn set_pc(&mut self, value: u64) {
        self.pc = value
    }

    fn push(&mut self, value: crate::inst::IntN) {
        self.writer.push(value.0, value.1);
    }

    fn insert(&mut self, value: crate::inst::IntN, offset: u8) {
        let addr = Aligned::new(self.pc as usize).unwrap();
        let current = self.bin.get_u32(addr).unwrap();
        let result = bit_write_u32(current, value.0, value.1, offset);
        self.bin.write_u32(addr, result);
    }

    fn begin_instr(&mut self) {
        self.writer = BitWriterU32::new();
    }

    fn end_instr(&mut self) {
        assert!(self.writer.all_bits_written());
        let value = self.writer.value();
        self.bin
            .write_u32(Aligned::new(self.pc as usize).unwrap(), value);
        self.pc += 4;
    }

    fn resolve_label(&mut self, key: Spur) -> Option<u64> {
        self.labels.addr_map.get(&key).copied()
    }

    fn push_label_fixup(&mut self, fixup: Fixup<Self, Spur, u64>) {
        self.labels.fixups.push(fixup);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ArgTest {
    Label(Spur),
}

impl Arg for ArgTest {
    fn kind(&self) -> operand::Kind {
        operand::Kind::Label
    }
    fn parse(&self) -> Result<operand::Ops, crate::inst::ParseError> {
        Ok(operand::Ops::Label(Spur::default()))
    }
}

pub fn process<'ast, 'src>(top: &'ast ast::Top<'ast, 'src>) {
    let args = &[ArgTest::Label(Spur::default())];
    let mnemonic = Mnemonic::from_str("B").unwrap();
    let mut instr = parse_variant(mnemonic, args.iter()).unwrap().unwrap();
    let mut e = Emit::new();
    let mut rl = LabelResolver::new();
    emit_instr(instr, &mut e);
}

#[cfg(test)]
mod tests {
    use std::hash::BuildHasherDefault;

    use lasso::Rodeo;

    use super::*;

    #[test]
    fn try_it() {
        let mut rodeo = Interner::with_hasher(BuildHasherDefault::default());
        let label = rodeo.get_or_intern_static("my_label");
        let args = &[ArgTest::Label(label)];
        let mnemonic = Mnemonic::from_str("B").unwrap();
        let instr = parse_variant(mnemonic, args.iter()).unwrap();
        let mut e = Emit::new();

        let addr = 0;
        e.set_pc(addr);
        
        emit_instr(instr.unwrap(), &mut e);
        e.labels.addr_map.insert(label, 16);
        
        println!(
            "0b{:032b}",
            e.bin
                .get_u32(Aligned::new(addr as usize).unwrap())
                .unwrap()
        );
        println!("{:?}", e.labels.fixups[0]);
        let fixup = e.labels.fixups[0].clone();
        apply_label_fixup(&mut e, fixup).unwrap();
        println!(
            "0b{:032b}",
            e.bin
                .get_u32(Aligned::new(addr as usize).unwrap())
                .unwrap()
        );
    }
}
