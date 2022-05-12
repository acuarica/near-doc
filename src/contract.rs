//! Allows the user to build a NEAR Rust Contract from multiple Rust source files.
use std::{collections::HashMap, ops::Deref};

use syn::{ImplItemMethod, Item, ItemImpl, ItemTrait, TraitItem, TraitItemMethod};

use crate::{NearImpl, NearMethod};

///
pub struct NearItemTrait {
    ///
    item_trait: ItemTrait,
    ///
    methods: HashMap<String, TraitItemMethod>,
}

impl NearItemTrait {
    ///
    pub fn get(&self, name: &String) -> Option<&TraitItemMethod> {
        self.methods.get(name)
    }
}

impl Deref for NearItemTrait {
    type Target = ItemTrait;

    fn deref(&self) -> &Self::Target {
        &self.item_trait
    }
}

impl NearItemTrait {
    fn new(item_trait: ItemTrait) -> Self {
        let mut methods = HashMap::new();
        for item in &item_trait.items {
            if let TraitItem::Method(method) = item {
                methods.insert(method.sig.ident.to_string(), method.clone());
            }
        }

        Self {
            item_trait,
            methods,
        }
    }
}

/// Represents a pass to several Rust files to build a NEAR Rust Contract.
pub struct Contract {
    /// Represents the name of the Contract to export.
    pub name: Option<String>,

    /// All trait definitions used in forward declarations.
    pub traits: HashMap<String, NearItemTrait>,

    /// Keeps track of `impl` items of the contract.
    pub interfaces: Vec<String>,

    ///
    pub methods: HashMap<String, (ImplItemMethod, ItemImpl)>,

    /// Keeps track of the `view_methods` in the contract.
    pub init_methods: Vec<String>,

    /// Keeps track of the `view_methods` in the contract.
    pub view_methods: Vec<String>,

    /// Keeps track of the `change_methods` in the contract.
    pub change_methods: Vec<String>,

    ///
    pub items: Vec<Item>,
}

impl Contract {
    /// Creates a new `Contract` instance with default values.
    pub fn new() -> Self {
        Self {
            name: None,
            traits: HashMap::new(),
            interfaces: Vec::new(),
            methods: HashMap::new(),
            init_methods: Vec::new(),
            view_methods: Vec::new(),
            change_methods: Vec::new(),
            items: Vec::new(),
        }
    }

    ///
    pub fn forward_traits(&mut self, items: &Vec<Item>) {
        for item in items {
            match item {
                Item::Impl(item_impl) => {
                    if let Some(methods) = item_impl.bindgen_methods() {
                        if let Some(trait_name) = item_impl.get_trait_name() {
                            self.interfaces.push(trait_name);
                        } else if let Some(impl_name) = item_impl.get_impl_name() {
                            self.name = Some(impl_name);
                        }

                        for method in methods {
                            let name = method.sig.ident.to_string();
                            self.methods
                                .insert(name.clone(), (method.clone(), item_impl.clone()));

                            if method.is_init() {
                                &mut self.init_methods
                            } else if method.is_mut() {
                                &mut self.change_methods
                            } else {
                                &mut self.view_methods
                            }
                            .push(name);
                        }
                    }
                }
                Item::Trait(item_trait) => self.push_trait(&item_trait),
                Item::Mod(item_mod) => {
                    if let Some((_, mod_items)) = &item_mod.content {
                        self.forward_traits(mod_items);
                    }
                }
                _ => {}
            }
        }
    }

    fn push_trait(&mut self, item_trait: &ItemTrait) {
        self.traits.insert(
            item_trait.ident.to_string(),
            NearItemTrait::new(item_trait.clone()),
        );
    }
}
