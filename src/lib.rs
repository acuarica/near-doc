//! Provides function to deal with Rust syntax.
#![deny(warnings)]
#![warn(missing_docs)]

pub mod ts;

use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};
use syn::{
    Attribute, FnArg, ImplItemMethod, Lit, Meta, MetaList, MetaNameValue, NestedMeta, Visibility,
};

/// Defines the `Args` to be used in binaries.
#[macro_export]
macro_rules! Args {
    ($bin_name:expr) => {
        #[derive(clap::Clap)]
        #[clap(name = $bin_name, version = env!("CARGO_PKG_VERSION"), author = env!("CARGO_PKG_AUTHORS"))]
        #[clap(setting = clap::AppSettings::ColoredHelp)]
        struct Args {
            /// Sets the time (any format) for generated output.
            #[clap(long = "now")]
            now: Option<String>,

            #[clap()]
            files: Vec<String>,
        }
        impl Args {
            fn now(&mut self) -> String {
                if self.now.is_none() {
                    self.now = Some(Utc::now().to_string());
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

/// Returns `true` if the `method` is explicitly marked as `pub`.
/// Returns `false` otherwise.
pub fn is_public(method: &ImplItemMethod) -> bool {
    match method.vis {
        Visibility::Public(_) => true,
        _ => false,
    }
}

/// Returns `true` if `attrs` contain `attr_name`.
/// Returns `false` otherwise.
pub fn has_attr(attrs: &Vec<Attribute>, attr_name: &str) -> bool {
    for attr in attrs {
        if attr.path.is_ident(attr_name) {
            return true;
        }
    }
    false
}

/// Returns whether the given `method` is marked as `payable`.
pub fn is_payable(method: &ImplItemMethod) -> bool {
    has_attr(&method.attrs, "payable")
}

/// Returns whether the given `method` is marked as `init`.
pub fn is_init(method: &ImplItemMethod) -> bool {
    has_attr(&method.attrs, "init")
}

/// Returns `true` if any of the attributes under item derive from `macro_name`.
/// Returns `false` otherwise.
pub fn derives(attrs: &Vec<Attribute>, macro_name: &str) -> bool {
    for attr in attrs {
        if attr.path.is_ident("derive") {
            if let Ok(Meta::List(MetaList { nested, .. })) = attr.parse_meta() {
                for elem in nested {
                    if let NestedMeta::Meta(meta) = elem {
                        if meta.path().is_ident(macro_name) {
                            return true;
                        }
                    }
                }
            } else {
                panic!("not expected");
            }
        }
    }
    false
}

/// Returns `true` if `method` is declared as `mut`.
pub fn is_mut(method: &ImplItemMethod) -> bool {
    if let Some(FnArg::Receiver(r)) = method.sig.inputs.iter().next() {
        r.mutability.is_some()
    } else {
        false
    }
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
