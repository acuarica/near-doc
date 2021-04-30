//! Provides function to deal with Rust syntax.
#![deny(warnings)]
#![warn(missing_docs)]

mod near_syn;
pub mod ts;

pub use crate::near_syn::*;

use std::io::Write;
use syn::{Attribute, Lit, Meta, MetaNameValue};

/// Joins segments of path by `::`.
pub fn join_path(path: &syn::Path) -> String {
    path.segments
        .iter()
        .map(|seg| seg.ident.to_string())
        .collect::<Vec<String>>()
        .join("::")
}

/// Writes Rust `doc` comments to `file`.
/// Each line of `doc` is prefixed with `prefix`.
pub fn write_docs<W: Write, F: Fn(String) -> String>(
    file: &mut W,
    attrs: &Vec<Attribute>,
    mapf: F,
) {
    for attr in attrs {
        if attr.path.is_ident("doc") {
            if let Ok(Meta::NameValue(MetaNameValue {
                lit: Lit::Str(lit), ..
            })) = attr.parse_meta()
            {
                writeln!(file, "{}", mapf(lit.value())).unwrap();
            } else {
                panic!("not expected");
            }
        }
    }
}
