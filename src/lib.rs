#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::if_not_else)]
#![feature(assert_matches)]
#![feature(ascii_char)]
#![feature(ascii_char_variants)]
#![feature(try_trait_v2)]
#![feature(never_type)]
#![feature(decl_macro)]
#![feature(lint_reasons)]
#![feature(macro_metavar_expr)]
#![feature(nonzero_ops)]

pub mod assembler;
pub mod probably;
pub mod scan;
pub mod stream;
