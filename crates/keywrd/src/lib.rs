#![feature(proc_macro_diagnostic)]
// TODO: temporary
#![allow(warnings)]

extern crate proc_macro;
use proc_macro::{
    token_stream::IntoIter, Delimiter, Group, Ident, Punct, Spacing, Span, TokenStream, TokenTree,
};

#[derive(Debug)]
pub(crate) enum Error {
    Duplicate,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl std::error::Error for Error {}
use Error::*;

type Result<T> = std::result::Result<T, Error>;
struct ResultAt<T>(Result<T>, Span);

fn unquote_str(str: String) -> Option<String> {
    if str.len() < 3 {
        return None;
    }
    if !str.starts_with('"') || !str.ends_with('"') {
        return None;
    }

    Some(String::from(&str[1..str.len() - 1]))
}

mod seq_tree {
    use super::{Error, Result};

    pub(crate) enum Node {
        Branch(u8, Vec<Node>),
        Leaf,
    }

    fn debug_fmt(
        nodes: &Vec<Node>,
        f: &mut std::fmt::Formatter<'_>,
        level: usize,
    ) -> std::fmt::Result {
        for node in nodes {
            writeln!(f)?;
            for _ in 0..level {
                write!(f, "|")?;
            }
            write!(f, "> ")?;
            match node {
                Node::Branch(value, children) => {
                    write!(f, "Branch: {:?},", *value as char)?;
                    debug_fmt(children, f, level + 1)?;
                }
                Node::Leaf => write!(f, "Leaf")?,
            }
        }
        Ok(())
    }

    pub(crate) struct Tree {
        root: Vec<Node>,
    }

    impl std::fmt::Debug for Tree {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            debug_fmt(&self.root, f, 0)
        }
    }

    impl Tree {
        pub fn new() -> Self {
            Self { root: Vec::new() }
        }
        pub fn insert<I: Iterator<Item = u8>>(&mut self, iter: I) -> Result<()> {
            Self::insert_rec(&mut self.root, iter)
        }
        pub fn is_empty(&self) -> bool {
            self.root.is_empty()
        }

        fn insert_rec<I: Iterator<Item = u8>>(nodes: &mut Vec<Node>, mut iter: I) -> Result<()> {
            match iter.next() {
                Some(value) => {
                    for node in nodes.iter_mut() {
                        if let Node::Branch(mid, children) = node {
                            if *mid == value {
                                return Self::insert_rec(children, iter);
                            }
                        }
                    }

                    let mut children: Vec<Node> = Vec::new();
                    Self::insert_rec(&mut children, iter);
                    nodes.push(Node::Branch(value, children));
                }
                None => {
                    if nodes.iter().any(|n| matches!(n, Node::Leaf)) {
                        return Err(Error::Duplicate);
                    }
                    nodes.push(Node::Leaf);
                }
            }
            Ok(())
        }
    }
}

fn collect_strs(stream: IntoIter) -> seq_tree::Tree {
    let mut tree = seq_tree::Tree::new();
    for token in stream {
        match token {
            TokenTree::Literal(lit) => match unquote_str(lit.to_string()) {
                Some(str) => {
                    tree.insert(str.bytes()).unwrap();
                }
                None => {
                    lit.span().error("unexpected token");
                }
            },
            _ => {
                token.span().error("unexpected token");
            }
        }
    }
    tree
}

fn gen_func_inner(tree: seq_tree::Tree) -> TokenStream {
    todo!()
}

fn gen_func(tree: seq_tree::Tree) -> TokenStream {
    let mut stream = "pub fn matcher<I: ::std::iter::Iterator<Item = u8>>(i: I) -> ()"
        .parse::<TokenStream>()
        .unwrap();

    stream.extend([TokenTree::Group(Group::new(
        Delimiter::Brace,
        gen_func_inner(tree),
    ))]);

    stream
}

#[proc_macro]
pub fn keywrd(item: TokenStream) -> TokenStream {
    let mut iter = item.into_iter();

    let tree = collect_strs(iter);

    TokenStream::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_str_tree_simple() {
        let mut tree = seq_tree::Tree::new();
        tree.insert("abc".bytes());
        tree.insert("abd".bytes());
        tree.insert("ab".bytes());
        tree.insert("abdd".bytes());
        println!("{:?}", tree);
    }
}
