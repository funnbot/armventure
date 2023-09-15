use std::io::{Error as IoError, IoSlice, Write};

use crate::sparsebin::SparseBin;

const MAGIC: [u8; 4] = [0x7f, b'E', b'L', b'F'];
#[derive(PartialEq, Eq)]
#[repr(u8)]
pub enum Class {
    Elf32 = 1,
    Elf64 = 2,
}
#[derive(PartialEq, Eq)]
#[repr(u8)]
pub enum Endian {
    Little = 1,
    Big = 2,
}
#[derive(Clone, Copy)]
#[repr(u16)]
pub enum Type {
    Reloc = 1,  // static object/library file
    Exec = 2,   // executable file
    Shared = 3, // dynamic object/library file
}
const ISA: u16 = 0xB7; // AArch64

macro_rules! copy_bytes_le {
    ($slice:expr, $int:expr) => {
        $slice.copy_from_slice(&($int).to_le_bytes())
    };
    ($slice:expr, $prim:ident, $int:expr) => {
        $slice.copy_from_slice(&($prim::try_from($int).unwrap()).to_le_bytes())
    };
    ($slice:ident, $addr:literal, u64, $expr:expr) => {
        $slice[$addr..$addr + 8].copy_from_slice(&(u64::try_from($expr).unwrap()).to_le_bytes());
    };
}

pub struct Header {
    pub entry: u64,
    pub ty: Type,
    pub prog_count: u16,
    pub sect_count: u16,
    pub prog_table_addr: u64,
    pub sect_table_addr: u64,
    pub sect_names_idx: u16,
}
impl Header {
    pub const SIZE_64: usize = 0x40;
    pub fn copy_data_64le(&self, d: &mut [u8]) {
        assert!(d.len() == Self::SIZE_64);
        d[0..0x4].copy_from_slice(&MAGIC);
        d[0x4] = Class::Elf64 as u8;
        d[0x5] = Endian::Little as u8;
        d[0x6] = 1; // version
        d[0x7] = 0x3; // linux abi
        d[0x8..0x10].fill(0); // pad
        d[0x10..0x12].copy_from_slice(&(self.ty as u16).to_le_bytes());
        d[0x12..0x14].copy_from_slice(&ISA.to_le_bytes());
        d[0x14..0x18].copy_from_slice(&1u32.to_le_bytes()); // version
        d[0x18..0x20].copy_from_slice(&self.entry.to_le_bytes());
        // program header table (starts after header)
        d[0x20..0x28].copy_from_slice(&self.prog_table_addr.to_le_bytes());
        // section header table (starts after program header)
        d[0x28..0x30].copy_from_slice(&self.sect_table_addr.to_le_bytes());
        // TODO: header[0x30..0x34] e_flags
        copy_bytes_le!(d[0x34..0x36], u16, Self::SIZE_64);
        copy_bytes_le!(d[0x36..0x38], u16, prog::Header::SIZE_64);
        d[0x38..0x3A].copy_from_slice(&self.prog_count.to_le_bytes());
        copy_bytes_le!(d[0x3A..0x3C], u16, sect::Header::SIZE_64);
        d[0x3C..0x3E].copy_from_slice(&self.sect_count.to_le_bytes());
        // index of the section header table entry that contains the section names
        d[0x3E..0x40].copy_from_slice(&self.sect_names_idx.to_le_bytes());
    }
}

// memory mapped segments
mod prog {
    #[derive(Clone, Copy)]
    #[repr(u32)]
    pub enum Type {
        Load = 1,
        Dynamic = 2,
        Interp = 3,
        Note = 4,
        HeaderTable = 6,
        TheadLocalStorage = 7,
    }
    pub mod flag {
        pub const EXEC: u32 = 1;
        pub const WRITE: u32 = 2;
        pub const READ: u32 = 4;
    }
    pub const fn flags(exec: bool, write: bool, read: bool) -> u32 {
        (exec as u32) | ((write as u32) << 1) | ((read as u32) << 2)
    }
    pub struct Header {
        pub ty: Type,
        pub flags: u32,
        pub file_addr: usize,
        pub virt_addr: usize,
        pub file_size: usize,
        pub virt_size: usize,
        pub align: usize,
    }
    impl Header {
        pub const SIZE_64: usize = 0x38;
        pub fn copy_data_64le(&self, d: &mut [u8]) {
            assert!(d.len() == Self::SIZE_64);
            assert!(self.align.is_power_of_two());
            copy_bytes_le!(d[0x0..0x4], self.ty as u32);
            copy_bytes_le!(d[0x4..0x8], self.flags);
            copy_bytes_le!(d[0x8..0x10], u64, self.file_addr);
            copy_bytes_le!(d[0x10..0x18], u64, self.virt_addr);
            // physical address (not used?)
            copy_bytes_le!(d[0x18..0x20], u64, 0);
            copy_bytes_le!(d[0x20..0x28], u64, self.file_size);
            copy_bytes_le!(d[0x28..0x30], u64, self.virt_size);
        }
    }
}

// static data sections
mod sect {
    #[derive(Clone, Copy)]
    #[repr(u32)]
    pub enum Type {
        ProgBits = 1,
        SymTab = 2,
        StrTab = 3,
        RelAdd = 4, // relocation (with addends)
        Hash = 5,
        Dynamic = 6,
        Note = 7,
        NoBits = 8, // bss
        Rel = 9,    // relocation (no addends)
        DynSym = 0xB,
        InitArray = 0xE,
        FInitArray = 0xF,
        PreInitArray = 0x10,
        Group = 0x11,
        SymTabExtIdx = 0x12,
    }
    pub mod flag {
        pub const WRITE: usize = 0x1;
        pub const ALLOC: usize = 0x2;
        pub const EXEC: usize = 0x4;
        pub const MERGE: usize = 0x10;
        pub const STRINGS: usize = 0x20;
        pub const INFO_LINK: usize = 0x40;
        pub const LINK_ORDER: usize = 0x80;
        pub const OS_NONCONFORMING: usize = 0x100;
        pub const GROUP: usize = 0x200;
        pub const TLS: usize = 0x400;
    }
    pub struct Header {
        pub name_offset: u32,
        pub ty: Type,
        pub flags: usize,
        pub virt_addr: usize,
        pub file_addr: usize,
        pub file_size: usize,
        pub link_idx: u32,
        pub info: u32,
        pub align: usize,
        pub entry_size: usize,
    }
    impl Header {
        pub const SIZE_64: usize = 0x40;
        pub fn copy_data_64le(&self, d: &mut [u8]) {
            assert!(d.len() == Self::SIZE_64);
            assert!(self.align.is_power_of_two());
            copy_bytes_le!(d[0x0..0x4], self.name_offset);
            copy_bytes_le!(d[0x4..0x8], self.ty as u32);
            copy_bytes_le!(d[0x8..0x10], u64, self.flags);
            copy_bytes_le!(d[0x10..0x18], u64, self.virt_addr);
            copy_bytes_le!(d[0x18..0x20], u64, self.file_addr);
            copy_bytes_le!(d[0x20..0x28], u64, self.file_size);
            copy_bytes_le!(d[0x28..0x2C], self.link_idx);
            copy_bytes_le!(d[0x2C..0x30], self.info);
            copy_bytes_le!(d[0x30..0x38], u64, self.align);
            copy_bytes_le!(d, 0x38, u64, self.entry_size);
        }
    }
}

pub struct Elf {
    entry: u64,
    ty: Type,
    prog_tab: Vec<prog::Header>,
    sect_tab: Vec<sect::Header>,
    bin: SparseBin,
}

impl Elf {
    pub fn new() -> Self {
        Self {
            entry: 0,
            ty: Type::Exec,
            prog_tab: Vec::new(),
            sect_tab: Vec::new(),
            bin: SparseBin::new(),
        }
    }

    pub fn write_64le_to<W: Write>(self, file: &mut W) -> Result<(), IoError> {
        let prog_offset: usize = Header::SIZE_64;
        let sect_offset: usize = prog_offset + prog::Header::SIZE_64 * self.prog_tab.len();
        let size = prog_offset + sect_offset + sect::Header::SIZE_64 * self.sect_tab.len();

        let mut data = Vec::<u8>::with_capacity(size);
        let header = Header {
            entry: self.entry,
            ty: self.ty,
            prog_count: self.prog_tab.len().try_into().unwrap(),
            sect_count: self.sect_tab.len().try_into().unwrap(),
            prog_table_addr: prog_offset as u64,
            sect_table_addr: sect_offset as u64,
            sect_names_idx: 0,
        };
        header.copy_data_64le(&mut data[0..Header::SIZE_64]);
        for (i, prog) in self.prog_tab.iter().enumerate() {
            let start = prog_offset + i * prog::Header::SIZE_64;
            let end = start + prog::Header::SIZE_64;
            prog.copy_data_64le(&mut data[start..end]);
        }
        for (i, sect) in self.sect_tab.iter().enumerate() {
            let start = sect_offset + i * sect::Header::SIZE_64;
            let end = start + sect::Header::SIZE_64;
            sect.copy_data_64le(&mut data[start..end]);
        }

        file.write_all(data.as_slice())?;

        Ok(())
    }
}
