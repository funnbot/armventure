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
#![feature(const_trait_impl)]
#![feature(let_chains)]
#![feature(associated_type_bounds)]
#![feature(pointer_is_aligned)]
//#![feature(generic_const_exprs)]

pub mod assembler;
//pub mod diag;
pub mod probably;
//pub mod scan;
pub mod inst;
pub mod num;
pub mod stream;

mod bitwriter;
mod code_file;
mod code_stream;
mod resolve;
mod sparsebin;
mod unwrap_macro;
