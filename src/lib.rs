// #![warn(clippy::pedantic)]
// #![warn(clippy::nursery)]
#![allow(clippy::if_not_else)]
#![allow(clippy::new_without_default)]
#![feature(assert_matches)]
#![feature(ascii_char)]
#![feature(ascii_char_variants)]
#![feature(try_trait_v2)]
#![feature(never_type)]
#![feature(lint_reasons)]
#![feature(macro_metavar_expr)]
#![feature(nonzero_ops)]
#![feature(char_indices_offset)]
#![feature(exhaustive_patterns)]
#![feature(let_chains)]
#![feature(associated_type_bounds)]
#![feature(pointer_is_aligned)]
#![feature(min_specialization)]
#![feature(unchecked_math)]
#![feature(test)]
#![feature(decl_macro)]
#![feature(associated_type_defaults)]
#![feature(effects)]
// #![feature(inherent_associated_types)]
// #![feature(generic_const_exprs)]
// #![feature(portable_simd)]
#![cfg_attr(test, allow(warnings))]

extern crate test;

pub mod assembler;
//pub mod diag;
pub mod probably;
//pub mod scan;
pub mod inst;
pub mod stream;

mod addr;
mod bitstack;
mod code;
mod code_stream;
mod elf;
mod enum_str;
mod intern;
mod sparsebin;
mod unwrap_macro;
