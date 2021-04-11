#![deny(warnings)]
// #![warn(missing_docs)]

pub mod ts;

use std::{fs::File, io::Read, path::Path};
use syn::{
    Attribute, FnArg, ImplItemMethod, Lit, Meta, MetaList, MetaNameValue, NestedMeta, Visibility,
};

/// Returns the Rust syntax tree for the given `file_name` path.
/// Panics if the file cannot be open or the file has syntax errors.
pub fn parse_rust<S: AsRef<Path>>(file_name: S) -> syn::File {
    let mut file = File::open(file_name).expect("Unable to open file");
    let mut src = String::new();
    file.read_to_string(&mut src).expect("Unable to read file");

    syn::parse_file(&src).expect("Unable to parse file")
}

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

pub fn has_attr(attrs: &Vec<Attribute>, attr_name: &str) -> bool {
    for attr in attrs {
        if attr.path.is_ident(attr_name) {
            return true;
        }
    }
    false
}

/// Returns `true` if the item has any derive attribute 
/// `macro_name`.
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

pub fn is_mut(method: &ImplItemMethod) -> bool {
    if let Some(FnArg::Receiver(r)) = method.sig.inputs.iter().next() {
        r.mutability.is_some()
    } else {
        false
    }
}

pub fn extract_docs(attrs: &Vec<Attribute>, indent: &str) {
    println!("{}/**", indent);
    for attr in attrs {
        if attr.path.is_ident("doc") {
            if let Ok(Meta::NameValue(MetaNameValue {
                lit: Lit::Str(lit), ..
            })) = attr.parse_meta()
            {
                println!("{} * {}", indent, lit.value());
            } else {
                panic!("not expected");
            }
        }
    }
    println!("{} */", indent);
}
