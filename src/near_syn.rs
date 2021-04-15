//! Augments `syn`'s AST with helper methods to deal with Near SDK definitions.

use syn::{
    Attribute, FnArg, ImplItem, ImplItemMethod, ItemImpl, ItemStruct, Meta, MetaList, NestedMeta,
    Visibility,
};

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

/// Defines helper methods to deal with Near `struct`s.
pub trait NearStruct {
    /// Returns whether the given `self` method derives `serde::Serialize`.
    fn is_serialize(&self) -> bool;

    /// Returns whether the given `self` method derives `serde::Deserialize`.
    fn is_deserialize(&self) -> bool;

    /// Returns whether the given `self` method derives either `serde::Serialize` or `serde::Deserialize`.
    fn is_serde(&self) -> bool;
}

impl NearStruct for ItemStruct {
    fn is_serialize(&self) -> bool {
        derives(&self.attrs, "Serialize")
    }

    fn is_deserialize(&self) -> bool {
        derives(&self.attrs, "Deserialize")
    }

    fn is_serde(&self) -> bool {
        self.is_serialize() || self.is_deserialize()
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
