//! Allows the user to build a NEAR Rust Contract from multiple Rust source files.
use std::{collections::HashMap, ops::Deref};

use syn::{
    Attribute, File, ImplItemMethod, Item, ItemEnum, ItemImpl, ItemStruct, ItemTrait, ItemType,
    TraitItem, TraitItemMethod,
};

use crate::near_sdk_syn::{NearBindgen, NearImpl, NearMethod, NearSerde};

/// Represents a pass to several Rust files to build a NEAR Rust Contract.
pub struct Contract {
    /// Represents the name of the Contract to export.
    /// `None` to represent this `Contract` has no name yet.
    /// A NEAR SDK Contract must have a name to be valid.
    pub name: Option<String>,

    ///
    pub top_level_attrs: Vec<Attribute>,

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
    pub items: Vec<NearItem>,
}

///
pub enum NearItem {
    ///
    Impl(ItemImpl),
    ///
    Struct(ItemStruct),
    ///
    Enum(ItemEnum),
    ///
    Type(ItemType),
}

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

impl Contract {
    /// Creates a new `Contract` instance with default values.
    pub fn new() -> Self {
        Self {
            name: None,
            top_level_attrs: Vec::new(),
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
    pub fn push_asts(&mut self, asts: Vec<File>) {
        for ast in asts {
            self.push_ast(ast);
        }
    }

    ///
    pub fn push_ast(&mut self, ast: File) {
        if self.push_items(ast.items) {
            self.top_level_attrs = ast.attrs;
        }
    }

    ///
    pub fn push_items(&mut self, items: Vec<Item>) -> bool {
        let mut declares_bindgen = false;
        for item in items {
            match item {
                Item::Impl(item_impl) => self.push_impl(item_impl),
                Item::Struct(item_struct) => {
                    if self.push_struct(item_struct) {
                        declares_bindgen = true;
                    }
                }
                Item::Enum(item_enum) => self.push_enum(item_enum),
                Item::Type(item_type) => self.push_typedef(item_type),
                Item::Trait(item_trait) => self.push_trait(&item_trait),
                Item::Mod(item_mod) => {
                    if let Some((_, mod_items)) = item_mod.content {
                        self.push_items(mod_items);
                    }
                }
                _ => {}
            }
        }

        declares_bindgen
    }

    fn push_impl(&mut self, item_impl: ItemImpl) {
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

            self.items.push(NearItem::Impl(item_impl));
        }
    }

    fn push_struct(&mut self, item_struct: ItemStruct) -> bool {
        if !item_struct.is_serde() {
            return false;
        }

        let is_bindgen = item_struct.is_bindgen();
        self.items.push(NearItem::Struct(item_struct));

        is_bindgen
    }

    fn push_enum(&mut self, item_enum: ItemEnum) {
        if !item_enum.is_serde() {
            return;
        }

        self.items.push(NearItem::Enum(item_enum));
    }

    fn push_typedef(&mut self, item_type: ItemType) {
        self.items.push(NearItem::Type(item_type));
    }

    fn push_trait(&mut self, item_trait: &ItemTrait) {
        self.traits.insert(
            item_trait.ident.to_string(),
            NearItemTrait::new(item_trait.clone()),
        );
    }
}
