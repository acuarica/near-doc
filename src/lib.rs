//! Provides function to deal with Rust syntax.
#![deny(warnings)]
#![warn(missing_docs)]

mod near_syn;
pub mod ts;

pub use crate::near_syn::*;

use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};
use syn::{Attribute, Lit, Meta, MetaNameValue};

/// Defines the `Args` to be used in binaries.
#[macro_export]
macro_rules! Args {
    ($bin_name:expr) => {
        #[derive(clap::Clap)]
        #[clap(name = $bin_name, version = env!("CARGO_PKG_VERSION"), author = env!("CARGO_PKG_AUTHORS"))]
        #[clap(setting = clap::AppSettings::ColoredHelp)]
        struct Args {

            /// Does not emit date/time information,
            /// otherwise current time is being emitted
            #[clap(long)]
            no_now: bool,

            #[clap()]
            files: Vec<String>,

            #[clap(skip)]
            now: Option<String>,
        }
        impl Args {
            fn now(&mut self) -> String {
                if self.now.is_none() {
                    self.now = Some(if self.no_now {"".to_string()} else {format!("on {}",chrono::Utc::now().to_string())});
                }
                self.now.clone().unwrap()
            }
        }
    }
}

/// Returns the Rust syntax tree for the given `file_name` path.
/// Panics if the file cannot be open or the file has syntax errors.
pub fn parse_rust<S: AsRef<Path>>(file_name: S) -> syn::File {
    let mut file = File::open(file_name).expect("Unable to open file");
    let mut src = String::new();
    file.read_to_string(&mut src).expect("Unable to read file");

    syn::parse_file(&src).expect("Unable to parse file")
}

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
