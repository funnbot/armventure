#![feature(proc_macro_diagnostic)]

extern crate proc_macro;
use proc_macro::{Delimiter, Group, Ident, Punct, Span, TokenStream, TokenTree};

use adt_internal_items;

enum Generic {
    Type(Ident),
    Lifetime(Ident),
}

struct TypeBound {
    lhs: TokenStream,
    rhs: TokenStream,
    used_generics: Vec<Generic>,
}

struct ModPath(Vec<Ident>);

struct Empty {}

struct Value {}

struct TupleStruct {}

struct Enum {
    fields: Vec<Item>,
}

struct Struct {
    fields: Vec<Item>,
}

enum ItemKind {
    Empty(Empty),
    Value(Value),
    TupleStruct(TupleStruct),
    Enum(Enum),
    Struct(Struct),
}

struct Item {
    name: Ident,
    info: ItemKind,
    used_generics: Vec<Generic>,
}

struct Adt {
    module: ModPath,
    used_generics: Vec<Generic>,
    repr: Item,
}

struct Attr {
    stream: TokenStream,
}

fn parse_attr(attr: TokenStream) -> Attr {
    Attr { stream: attr }
}

fn parse_item(stream: TokenStream) -> Adt {
    let mut iter = stream.into_iter();
    let mut attrs: Vec<Attr> = Vec::new();
    while let Some(token) = iter.next() {
        match token {
            TokenTree::Group(group) => {}
            TokenTree::Punct(punct) => match punct.as_char() {
                '#' => match iter.next() {
                    Some(TokenTree::Group(group)) => {
                        if group.delimiter() == Delimiter::Bracket {
                            attrs.push(parse_attr(group.stream()));
                        } else {
                            todo!("error if # isnt followed by [ .. ] group")
                        }
                    }
                    _ => {
                        todo!("error if # isnt followed by group")
                    }
                },
                _ => {}
            },
            TokenTree::Ident(ident) => {
                
            }
            _ => {}
        }
    }
    todo!()
}

fn parse(attr: TokenStream, item: TokenStream) -> Adt {
    todo!()
}

#[proc_macro_attribute]
pub fn adt(attr: TokenStream, item: TokenStream) -> TokenStream {
    TokenStream::new()
}

// #[algebraic::adt(
// attr =    in foo, as foo
//  )]
// item = the rest
// #[derive(Debug)]
// enum Hello {
//     Value = 1,
// }
