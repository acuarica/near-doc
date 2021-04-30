//! Augments `syn`'s AST with helper methods to deal with Near SDK definitions.
//! Additionally, provides function to deal with Rust syntax.
#![deny(warnings)]
#![warn(missing_docs)]

use syn::{
    Attribute, FnArg, ImplItem, ImplItemMethod, ItemEnum, ItemImpl, ItemStruct, Lit, Meta,
    MetaList, MetaNameValue, NestedMeta, Visibility,
};

pub mod ts;

/// Defines standard attributes found in the Near SDK.
pub trait NearImpl {
    /// Returns whether the given `self` implementation is marked as `near_bindgen`.
    fn is_bindgen(&self) -> bool;

    /// Returns whether the given `self` implementation has any exported method.
    fn has_exported_methods(&self) -> bool;
}

impl NearImpl for ItemImpl {
    fn is_bindgen(&self) -> bool {
        has_attr(&self.attrs, "near_bindgen")
    }

    fn has_exported_methods(&self) -> bool {
        for impl_item in self.items.iter() {
            if let ImplItem::Method(method) = impl_item {
                if method.is_exported(self) {
                    return true;
                }
            }
        }
        false
    }
}

/// Defines standard attributes and helper methods used when exporting a contract.
pub trait NearMethod {
    /// Returns whether the given `self` method is declared as `pub`.
    fn is_public(&self) -> bool;

    /// Returns whether the given `self` method is declared as `mut`.
    fn is_mut(&self) -> bool;

    /// Returns whether the given `self` method is marked as `init`.
    fn is_init(&self) -> bool;

    /// Returns whether the given `self` method is marked as `payable`.
    fn is_payable(&self) -> bool;

    /// Returns whether the given `self` method is marked as `private`.
    fn is_private(&self) -> bool;

    /// Returns whether the given `self` method in `input` impl is being exported.
    fn is_exported(&self, input: &ItemImpl) -> bool;
}

impl NearMethod for ImplItemMethod {
    fn is_public(self: &ImplItemMethod) -> bool {
        match self.vis {
            Visibility::Public(_) => true,
            _ => false,
        }
    }

    fn is_mut(&self) -> bool {
        if let Some(FnArg::Receiver(r)) = self.sig.inputs.iter().next() {
            r.mutability.is_some()
        } else {
            false
        }
    }

    fn is_init(&self) -> bool {
        has_attr(&self.attrs, "init")
    }

    fn is_payable(&self) -> bool {
        has_attr(&self.attrs, "payable")
    }

    fn is_private(self: &ImplItemMethod) -> bool {
        has_attr(&self.attrs, "private")
    }

    fn is_exported(&self, input: &ItemImpl) -> bool {
        (self.is_public() || input.trait_.is_some()) && !self.is_private()
    }
}

/// Defines methods to deal with serde's declarations in `struct`s or `enum`s.
pub trait NearSerde {
    /// Returns whether the given `self` item derives `serde::Serialize`.
    fn is_serialize(&self) -> bool;

    /// Returns whether the given `self` item derives `serde::Deserialize`.
    fn is_deserialize(&self) -> bool;

    /// Returns whether the given `self` item derives either `serde::Serialize` or `serde::Deserialize`.
    fn is_serde(&self) -> bool;
}

impl<I: NearAttributable> NearSerde for I {
    fn is_serialize(&self) -> bool {
        derives(&self.attrs(), "Serialize")
    }

    fn is_deserialize(&self) -> bool {
        derives(&self.attrs(), "Deserialize")
    }

    fn is_serde(&self) -> bool {
        self.is_serialize() || self.is_deserialize()
    }
}

/// Any Rust item, *e.g.*, `struct` or `enum` to which attributes can attached to.
pub trait NearAttributable {
    /// The attributes of this item.
    fn attrs(&self) -> &Vec<Attribute>;
}

impl NearAttributable for ItemStruct {
    fn attrs(&self) -> &Vec<Attribute> {
        &self.attrs
    }
}

impl NearAttributable for ItemEnum {
    fn attrs(&self) -> &Vec<Attribute> {
        &self.attrs
    }
}

/// Returns `true` if `attrs` contain `attr_name`.
/// Returns `false` otherwise.
fn has_attr(attrs: &Vec<Attribute>, attr_name: &str) -> bool {
    for attr in attrs {
        if attr.path.is_ident(attr_name) {
            return true;
        }
    }
    false
}

/// Returns `true` if any of the attributes under item derive from `macro_name`.
/// Returns `false` otherwise.
fn derives(attrs: &Vec<Attribute>, macro_name: &str) -> bool {
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
pub fn write_docs<W: std::io::Write, F: Fn(String) -> String>(
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
