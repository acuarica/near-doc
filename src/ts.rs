//! Functions to transpile Rust to TypeScript.

use crate::{join_path, near_syn::NearMethod, write_docs, NearImpl, NearSerde};
use std::ops::Deref;
use syn::{
    Attribute, Fields, ImplItem, ImplItemMethod, Item, ItemEnum, ItemImpl, ItemStruct,
    PathArguments, ReturnType, Type,
};

/// Represents a pass to several Rust files to generate TypeScript bindings.
pub struct TS<T> {
    /// Represents the name of the contract to export.
    pub name: String,
    impl_count: u32,
    /// interfaces
    pub interfaces: Vec<String>,
    /// view
    pub view_methods: Vec<String>,
    /// change
    pub change_methods: Vec<String>,
    /// Output buffer where to store the generated TypeScript bindings.
    pub buf: T,
}

macro_rules! ln {
    ($this:ident, $($arg:tt)*) => ({
        writeln!($this.buf, $($arg)*).unwrap()
    })
}

impl<T: std::io::Write> TS<T> {
    /// Creates a new `TS` instance.
    ///
    /// ```
    /// let mut ts = near_syn::ts::TS::new(Vec::new());
    /// assert_eq!(String::from_utf8_lossy(&ts.buf), "");
    /// ```
    pub fn new(buf: T) -> Self {
        Self {
            name: String::new(),
            impl_count: 0,
            interfaces: Vec::new(),
            view_methods: Vec::new(),
            change_methods: Vec::new(),
            buf,
        }
    }

    /// Exports common Near types.
    ///
    /// ```
    /// let mut ts = near_syn::ts::TS::new(Vec::new());
    /// ts.ts_prelude(" 2021".to_string(), "bin");
    /// assert_eq!(String::from_utf8_lossy(&ts.buf), format!(
    /// r#"// TypeScript bindings generated with bin v{} {} 2021
    ///
    /// // Exports common NEAR Rust SDK types
    /// export type U64 = string;
    /// export type I64 = string;
    /// export type U128 = string;
    /// export type I128 = string;
    /// export type AccountId = string;
    /// export type ValidAccountId = string;
    ///
    /// "#,
    ///   env!("CARGO_PKG_VERSION"),
    ///   env!("CARGO_PKG_REPOSITORY"),
    ///   ));
    /// ```
    pub fn ts_prelude(&mut self, now: String, bin_name: &str) {
        ln!(
            self,
            "// TypeScript bindings generated with {} v{} {}{}\n",
            bin_name,
            env!("CARGO_PKG_VERSION"),
            env!("CARGO_PKG_REPOSITORY"),
            now
        );

        ln!(self, "// Exports common NEAR Rust SDK types");
        ln!(self, "export type U64 = string;");
        ln!(self, "export type I64 = string;");
        ln!(self, "export type U128 = string;");
        ln!(self, "export type I128 = string;");
        ln!(self, "export type AccountId = string;");
        ln!(self, "export type ValidAccountId = string;");
        ln!(self, "");
    }

    /// Exports the main type implemented by the contract.
    /// The `name` and `interfaces` must be set in order to export the main type.
    ///
    /// ```
    /// let mut ts = near_syn::ts::TS::new(Vec::new());
    /// ts.name = "Contract".to_string();
    /// ts.interfaces.push("Self0".to_string());
    /// ts.ts_main_type();
    /// assert_eq!(String::from_utf8_lossy(&ts.buf), "export type Contract = Self0;\n\n");
    /// ```
    pub fn ts_main_type(&mut self) {
        if !self.name.is_empty() && !self.interfaces.is_empty() {
            ln!(
                self,
                "export type {} = {};\n",
                self.name,
                self.interfaces.join(" & ")
            );
        }
    }

    /// Exports the methods object required by `near-api-js` to be able
    /// to find contract methods.
    ///
    /// ```
    /// let mut ts = near_syn::ts::TS::new(Vec::new());
    /// ts.name = "Contract".to_string();
    /// ts.view_methods.push("get".to_string());
    /// ts.change_methods.push("set".to_string());
    /// ts.change_methods.push("insert".to_string());
    /// ts.ts_contract_methods();
    /// assert_eq!(String::from_utf8_lossy(&ts.buf),
    /// r#"export const ContractMethods = {
    ///     viewMethods: [
    ///         "get",
    ///     ],
    ///     changeMethods: [
    ///         "set",
    ///         "insert",
    ///     ],
    /// };
    /// "#);
    /// ```
    ///
    /// Both `viewMethods` and `changeMethods` fields must be present in the
    /// resulting object.
    /// This is required by `near-api-js`.
    /// For example,
    ///
    /// ```
    /// let mut ts = near_syn::ts::TS::new(Vec::new());
    /// ts.name = "Contract".to_string();
    /// ts.view_methods.push("get".to_string());
    /// ts.ts_contract_methods();
    /// assert_eq!(String::from_utf8_lossy(&ts.buf),
    /// r#"export const ContractMethods = {
    ///     viewMethods: [
    ///         "get",
    ///     ],
    ///     changeMethods: [
    ///     ],
    /// };
    /// "#);
    /// ```
    pub fn ts_contract_methods(&mut self) {
        fn fmt(methods: &Vec<String>) -> String {
            methods
                .iter()
                .map(|m| format!("        {:?},\n", m))
                .collect::<Vec<String>>()
                .join("")
        }

        ln!(self, "export const {}Methods = {{", self.name);
        ln!(
            self,
            "    viewMethods: [\n{}    ],",
            fmt(&self.view_methods)
        );
        ln!(
            self,
            "    changeMethods: [\n{}    ],",
            fmt(&self.change_methods)
        );
        ln!(self, "}};");
    }

    /// Translates a collection of Rust items to TypeScript.
    /// It currently translates `type`, `struct`, `enum` and `impl` items to TypeScript.
    /// It traverses recursively `mod` definitions with braced content.
    /// The inner `mod`' items are flatten into a single TypeScript module.
    /// If an item in `items` is not one of the mentioned above, it is ignored.
    ///
    /// In order to translate a Rust unit file, use the `items` field.
    /// For example
    ///
    /// ```no_run
    /// let mut ts = near_syn::ts::TS::new(std::io::stdout());
    /// let ast = near_syn::parse_rust("path/to/file.rs");
    /// ts.ts_items(&ast.items);
    /// ```
    ///
    /// Notice how `mod` definitions are flattened:
    ///
    /// ```
    /// let mut ts = near_syn::ts::TS::new(Vec::new());
    /// let ast: syn::File = syn::parse2(quote::quote! {
    ///         /// Doc-comments are translated.
    ///         type T = u64;
    ///         mod inner_mod {
    ///             type S = u64;
    ///         }
    ///     }).unwrap();
    /// ts.ts_items(&ast.items);
    /// assert_eq!(String::from_utf8_lossy(&ts.buf),
    /// r#"/**
    ///  *  Doc-comments are translated.
    ///  */
    /// export type T = number;
    ///
    /// /**
    ///  */
    /// export type S = number;
    ///
    /// "#);
    /// ```
    pub fn ts_items(&mut self, items: &Vec<Item>) {
        for item in items {
            match item {
                Item::Type(item_type) => self.ts_type(&item_type),
                Item::Struct(item_struct) => self.ts_struct(&item_struct),
                Item::Enum(item_enum) => self.ts_enum(&item_enum),
                Item::Impl(item_impl) => self.ts_impl(&item_impl),
                Item::Mod(item_mod) => {
                    if let Some((_, mod_items)) = &item_mod.content {
                        self.ts_items(mod_items);
                    }
                }
                _ => {}
            }
        }
    }

    /// Translates a type alias to another type alias in TypeScript.
    ///
    /// ```
    /// let mut ts = near_syn::ts::TS::new(Vec::new());
    /// ts.ts_type(&syn::parse2(quote::quote! {
    ///         /// Doc-comments are translated.
    ///         type T = u64;
    ///     }).unwrap());
    /// assert_eq!(String::from_utf8_lossy(&ts.buf),
    /// r#"/**
    ///  *  Doc-comments are translated.
    ///  */
    /// export type T = number;
    ///
    /// "#);
    /// ```
    ///
    /// If doc-comments are omitted,
    /// TypeScript empty doc-comments will be emitted.
    ///
    /// ```
    /// let mut ts = near_syn::ts::TS::new(Vec::new());
    /// ts.ts_type(&syn::parse2(quote::quote! {
    ///         type T = u64;
    ///     }).unwrap());
    /// assert_eq!(String::from_utf8_lossy(&ts.buf),
    /// r#"/**
    ///  */
    /// export type T = number;
    ///
    /// "#);
    /// ```
    pub fn ts_type(&mut self, item_type: &syn::ItemType) {
        self.ts_doc(&item_type.attrs, "");
        ln!(
            self,
            "export type {} = {};",
            item_type.ident,
            ts_type(&item_type.ty)
        );
        ln!(self, "");
    }

    /// Generates the corresponding TypeScript bindings for the given `struct`.
    /// Doc-comments embedded in the Rust source file are included in the bindings.
    /// The `struct` must derive `Serialize` from `serde` in order to
    /// generate its corresponding TypeScript bindings.
    ///
    /// ```
    /// let mut ts = near_syn::ts::TS::new(Vec::new());
    /// ts.ts_struct(&syn::parse2(quote::quote! {
    ///         /// Doc-comments are also translated.
    ///         #[derive(Serialize)]
    ///         struct A {
    ///             /// Doc-comments here are translated as well.
    ///             field: u32,
    ///         }
    ///     }).unwrap());
    /// assert_eq!(String::from_utf8_lossy(&ts.buf),
    /// r#"/**
    ///  *  Doc-comments are also translated.
    ///  */
    /// export interface A {
    ///     /**
    ///      *  Doc-comments here are translated as well.
    ///      */
    ///     field: number;
    ///
    /// }
    ///
    /// "#);
    /// ```
    ///
    /// Single-compoenent tuple-structs are converted to TypeScript type synonym.
    ///
    /// ```
    /// let mut ts = near_syn::ts::TS::new(Vec::new());
    /// ts.ts_struct(&syn::parse2(quote::quote! {
    ///         /// Tuple struct with one component.
    ///         #[derive(Serialize)]
    ///         struct T(String);
    ///     }).unwrap());
    /// assert_eq!(String::from_utf8_lossy(&ts.buf),
    /// r#"/**
    ///  *  Tuple struct with one component.
    ///  */
    /// export type T = string;
    ///
    /// "#);
    /// ```
    ///
    /// On the other hand,
    /// tuple-structs with more than one component,
    /// are converted to TypeScript proper tuples.
    ///
    /// ```
    /// let mut ts = near_syn::ts::TS::new(Vec::new());
    /// ts.ts_struct(&syn::parse2(quote::quote! {
    ///         /// Tuple struct with one component.
    ///         #[derive(Serialize)]
    ///         struct T(String, u32);
    ///     }).unwrap());
    /// assert_eq!(String::from_utf8_lossy(&ts.buf),
    /// r#"/**
    ///  *  Tuple struct with one component.
    ///  */
    /// export type T = [string, number];
    ///
    /// "#);
    /// ```
    ///
    /// If derive `Serialize` is not found, given `struct` is omitted.
    ///
    /// ```
    /// let mut ts = near_syn::ts::TS::new(Vec::new());
    /// ts.ts_struct(&syn::parse2(quote::quote! {
    ///         struct A { }
    ///     }).unwrap());
    /// assert_eq!(String::from_utf8_lossy(&ts.buf), "");
    /// ```
    pub fn ts_struct(&mut self, item_struct: &ItemStruct) {
        if !item_struct.is_serde() {
            return;
        }

        self.ts_doc(&item_struct.attrs, "");
        match &item_struct.fields {
            Fields::Named(fields) => {
                ln!(self, "export interface {} {{", item_struct.ident);
                for field in &fields.named {
                    let field_name = field.ident.as_ref().unwrap();
                    let ty = ts_type(&field.ty);
                    self.ts_doc(&field.attrs, "    ");
                    ln!(self, "    {}: {};\n", field_name, ty);
                }
                ln!(self, "}}");
                ln!(self, "");
            }
            Fields::Unnamed(fields) => {
                let mut tys = Vec::new();
                for field in &fields.unnamed {
                    let ty = ts_type(&field.ty);
                    tys.push(ty);
                }
                ln!(
                    self,
                    "export type {} = {};\n",
                    item_struct.ident,
                    if tys.len() == 1 {
                        tys.get(0).unwrap().clone()
                    } else {
                        format!("[{}]", tys.join(", "))
                    }
                );
            }
            Fields::Unit => panic!("unit struct no supported"),
        }
    }

    /// Translates an enum to a TypeScript `enum` or `type` according to the
    /// Rust definition.
    /// The Rust `enum` must derive `Serialize` from `serde` in order
    /// to be translated.
    ///
    /// For instance, a plain Rust `enum` will be translated to an `enum`.
    ///
    /// ```
    /// let mut ts = near_syn::ts::TS::new(Vec::new());
    /// ts.ts_enum(&syn::parse2(quote::quote! {
    ///         /// Doc-comments are translated.
    ///         #[derive(Serialize)]
    ///         enum E {
    ///             /// Doc-comments here are translated as well.
    ///             V1,
    ///         }
    ///     }).unwrap());
    /// assert_eq!(String::from_utf8_lossy(&ts.buf),
    /// r#"/**
    ///  *  Doc-comments are translated.
    ///  */
    /// export enum E {
    ///     /**
    ///      *  Doc-comments here are translated as well.
    ///      */
    ///     V1,
    ///
    /// }
    ///
    /// "#);
    /// ```
    pub fn ts_enum(&mut self, item_enum: &ItemEnum) {
        if !item_enum.is_serde() {
            return;
        }

        self.ts_doc(&item_enum.attrs, "");
        ln!(self, "export enum {} {{", item_enum.ident);
        for variant in &item_enum.variants {
            self.ts_doc(&variant.attrs, "    ");
            ln!(self, "    {},\n", variant.ident);
        }
        ln!(self, "}}\n");
    }

    /// Translates an `impl` section to a TypeScript `interface.`
    ///
    /// A `struct` can have multiple `impl` sections with no `trait` to declare additional methods.
    /// Such an `impl` section is exported as `Self<number>`,
    /// where *number* is the `impl` section occurence.
    ///
    /// ```
    /// let mut ts = near_syn::ts::TS::new(Vec::new());
    /// ts.ts_impl(&syn::parse2(quote::quote! {
    ///         /// Doc-comments are translated.
    ///         #[near_bindgen]
    ///         impl Contract {
    ///             /// Doc-comments here are translated as well.
    ///             pub fn get(&self) -> u32 { 42 }
    ///         }
    ///     }).unwrap());
    /// ts.ts_main_type();
    /// assert_eq!(String::from_utf8_lossy(&ts.buf),
    /// r#"/**
    ///  *  Doc-comments are translated.
    ///  */
    /// export interface Self0 {
    ///     /**
    ///      *  Doc-comments here are translated as well.
    ///      */
    ///     get(): Promise<number>;
    ///
    /// }
    ///
    /// export type Contract = Self0;
    ///
    /// "#);
    /// ```
    pub fn ts_impl(&mut self, item_impl: &ItemImpl) {
        if !item_impl.is_bindgen() || !item_impl.has_exported_methods() {
            return;
        }

        self.ts_doc(&item_impl.attrs, "");
        if let Some((_, trait_path, _)) = &item_impl.trait_ {
            let trait_name = join_path(trait_path);
            self.interfaces.push(trait_name.clone());
            ln!(self, "export interface {} {{", trait_name);
        } else {
            if let syn::Type::Path(type_path) = &*item_impl.self_ty {
                let impl_name = format!("Self{}", self.impl_count);
                self.impl_count += 1;
                self.name = join_path(&type_path.path);
                self.interfaces.push(impl_name.clone());
                ln!(self, "export interface {} {{", impl_name);
            } else {
                panic!("name not found")
            }
        }

        {
            for item in item_impl.items.iter() {
                if let ImplItem::Method(method) = item {
                    if method.is_exported(item_impl) {
                        if !method.is_init() {
                            if method.is_mut() {
                                &mut self.change_methods
                            } else {
                                &mut self.view_methods
                            }
                            .push(method.sig.ident.to_string());
                        }
                        self.ts_doc(&method.attrs, "    ");
                        ln!(self, "    {}\n", ts_sig(&method));
                    }
                }
            }
        }

        ln!(self, "}}\n");
    }

    fn ts_doc(&mut self, attrs: &Vec<Attribute>, indent: &str) {
        ln!(self, "{}/**", indent);
        write_docs(&mut self.buf, attrs, |l| format!("{} * {}", indent, l));
        ln!(self, "{} */", indent);
    }
}

/// Return the TypeScript equivalent type of the Rust type represented by `ty`.
/// Rust primitives types and `String` are included.
///
/// ```
/// use syn::parse_str;
/// use near_syn::ts::ts_type;
///
/// assert_eq!(ts_type(&parse_str("bool").unwrap()), "boolean");
/// assert_eq!(ts_type(&parse_str("i8").unwrap()), "number");
/// assert_eq!(ts_type(&parse_str("u8").unwrap()), "number");
/// assert_eq!(ts_type(&parse_str("i16").unwrap()), "number");
/// assert_eq!(ts_type(&parse_str("u16").unwrap()), "number");
/// assert_eq!(ts_type(&parse_str("i32").unwrap()), "number");
/// assert_eq!(ts_type(&parse_str("u32").unwrap()), "number");
/// assert_eq!(ts_type(&parse_str("String").unwrap()), "string");
/// ```
///
/// Rust standard and collections types, *e.g.*, `Option`, `Vec` and `HashMap`,
/// are included in the translation.
///
/// ```
/// # use syn::parse_str;
/// # use near_syn::ts::ts_type;
/// assert_eq!(ts_type(&parse_str("Option<U64>").unwrap()), "U64|null");
/// assert_eq!(ts_type(&parse_str("Option<String>").unwrap()), "string|null");
/// assert_eq!(ts_type(&parse_str("Vec<ValidAccountId>").unwrap()), "ValidAccountId[]");
/// assert_eq!(ts_type(&parse_str("HashSet<ValidAccountId>").unwrap()), "ValidAccountId[]");
/// assert_eq!(ts_type(&parse_str("BTreeSet<ValidAccountId>").unwrap()), "ValidAccountId[]");
/// assert_eq!(ts_type(&parse_str("HashMap<AccountId, U128>").unwrap()), "Record<AccountId, U128>");
/// assert_eq!(ts_type(&parse_str("BTreeMap<AccountId, U128>").unwrap()), "Record<AccountId, U128>");
/// ```
///
/// Rust nested types are converted to TypeScript as well.
///
/// ```
/// # use syn::parse_str;
/// # use near_syn::ts::ts_type;
/// assert_eq!(ts_type(&parse_str("HashMap<AccountId, Vec<U128>>").unwrap()), "Record<AccountId, U128[]>");
/// assert_eq!(ts_type(&parse_str("Vec<Option<U128>>").unwrap()), "(U128|null)[]");
/// assert_eq!(ts_type(&parse_str("Option<Vec<U128>>").unwrap()), "U128[]|null");
/// assert_eq!(ts_type(&parse_str("Option<Option<U64>>").unwrap()), "U64|null|null");
/// assert_eq!(ts_type(&parse_str("Vec<Vec<U64>>").unwrap()), "U64[][]");
/// assert_eq!(ts_type(&parse_str("(U64)").unwrap()), "U64");
/// assert_eq!(ts_type(&parse_str("(U64, String, Vec<u32>)").unwrap()), "[U64, string, number[]]");
///
/// assert_eq!(ts_type(&parse_str("()").unwrap()), "void");
/// // assert_eq!(ts_type(&parse_str("std::vec::Vec<U64>").unwrap()), "U64[]");
/// ```
///
/// ## Panics
///
/// Panics when standard library generics types are used incorrectly.
/// For example `Option` or `HashMap<U64>`.
/// This situation can only happen on Rust source files that were **not** type-checked by `rustc`.
pub fn ts_type(ty: &Type) -> String {
    #[derive(PartialEq, PartialOrd)]
    enum Assoc {
        Single,
        Vec,
        Or,
    }
    fn single(ts: &str) -> (String, Assoc) {
        (ts.to_string(), Assoc::Single)
    }
    fn use_paren(ta: (String, Assoc), assoc: Assoc) -> String {
        if ta.1 > assoc {
            format!("({})", ta.0)
        } else {
            ta.0
        }
    }
    fn gen_args<'a>(p: &'a syn::TypePath, nargs: usize, name: &str) -> Vec<&'a Type> {
        if let PathArguments::AngleBracketed(args) = &p.path.segments[0].arguments {
            if args.args.len() != nargs {
                panic!(
                    "{} expects {} generic(s) argument(s), found {}",
                    name,
                    nargs,
                    args.args.len()
                );
            }
            let mut result = Vec::new();
            for arg in &args.args {
                if let syn::GenericArgument::Type(tk) = arg {
                    result.push(tk);
                } else {
                    panic!("No type provided for {}", name);
                }
            }
            result
        } else {
            panic!("{} used with no generic arguments", name);
        }
    }

    fn ts_type_assoc(ty: &Type) -> (String, Assoc) {
        match ty {
            Type::Path(p) => match crate::join_path(&p.path).as_str() {
                "bool" => single("boolean"),
                "u64" => single("number"),
                "i8" | "u8" | "i16" | "u16" | "i32" | "u32" => single("number"),
                "String" => single("string"),
                "Option" => {
                    let targs = gen_args(p, 1, "Option");
                    let ta = ts_type_assoc(&targs[0]);
                    (format!("{}|null", use_paren(ta, Assoc::Or)), Assoc::Or)
                }
                "Vec" | "HashSet" | "BTreeSet" => {
                    let targs = gen_args(p, 1, "Vec");
                    let ta = ts_type_assoc(&targs[0]);
                    (format!("{}[]", use_paren(ta, Assoc::Vec)), Assoc::Vec)
                }
                "HashMap" | "BTreeMap" => {
                    let targs = gen_args(p, 2, "HashMap");
                    let (tks, _) = ts_type_assoc(&targs[0]);
                    let (tvs, _) = ts_type_assoc(&targs[1]);
                    (format!("Record<{}, {}>", tks, tvs), Assoc::Single)
                }
                s => single(s),
            },
            Type::Paren(paren) => ts_type_assoc(paren.elem.as_ref()),
            Type::Tuple(tuple) => {
                if tuple.elems.is_empty() {
                    ("void".into(), Assoc::Single)
                } else {
                    let mut tys = Vec::new();
                    for elem_type in &tuple.elems {
                        let (t, _) = ts_type_assoc(&elem_type);
                        tys.push(t);
                    }
                    (format!("[{}]", tys.join(", ")), Assoc::Single)
                }
            }
            _ => panic!("type not supported"),
        }
    }
    ts_type_assoc(ty).0
}

/// Returns the signature of the given Rust `method`.
/// The resulting TypeScript binding is a valid method definition expected by the NEAR RPC.
/// Thus, the following conversion are applied:
/// - Function arguments are packed into a single TypeScript object argument
/// - Return type is wrapped into a `Promise`
/// - Types are converted using `ts_type`
///
/// ## Examples
///
/// ```
/// use syn::parse_str;
/// use near_syn::ts::ts_sig;
///
/// assert_eq!(ts_sig(&parse_str("fn a() {}").unwrap()), "a(): Promise<void>;");
/// assert_eq!(ts_sig(&parse_str("fn b(x: U128) {}").unwrap()), "b(args: { x: U128 }): Promise<void>;");
/// assert_eq!(ts_sig(&parse_str("fn c(x: U128, y: String) -> Vec<Token> {}").unwrap()), "c(args: { x: U128, y: string }): Promise<Token[]>;");
/// assert_eq!(ts_sig(&parse_str("fn d(x: U128, y: String, z: Option<U64>) -> Vec<Token> {}").unwrap()), "d(args: { x: U128, y: string, z: U64|null }): Promise<Token[]>;");
/// assert_eq!(ts_sig(&parse_str("fn e(x: U128) -> () {}").unwrap()), "e(args: { x: U128 }): Promise<void>;");
/// assert_eq!(ts_sig(&parse_str("fn f(paren: (String)) {}").unwrap()), "f(args: { paren: string }): Promise<void>;");
/// assert_eq!(ts_sig(&parse_str("fn get(&self) -> u32 {}").unwrap()), "get(): Promise<number>;");
/// assert_eq!(ts_sig(&parse_str("fn set(&mut self) {}").unwrap()), "set(gas?: any): Promise<void>;");
/// assert_eq!(ts_sig(&parse_str("fn set_args(&mut self, x: u32) {}").unwrap()), "set_args(args: { x: number }, gas?: any): Promise<void>;");
/// assert_eq!(ts_sig(&parse_str("fn a() -> Promise {}").unwrap()), "a(): Promise<void>;");
/// ```
pub fn ts_sig(method: &ImplItemMethod) -> String {
    let mut args = Vec::new();
    for arg in method.sig.inputs.iter() {
        match arg {
            syn::FnArg::Typed(pat_type) => {
                if let syn::Pat::Ident(pat_ident) = pat_type.pat.deref() {
                    let type_name = ts_type(&pat_type.ty);
                    let arg_ident = &pat_ident.ident;
                    args.push(format!("{}: {}", arg_ident, type_name));
                }
            }
            _ => {}
        }
    }

    if method.is_init() {
        format!("{}: {{ {} }};", method.sig.ident, args.join(", "),)
    } else {
        let ret_type = match &method.sig.output {
            ReturnType::Default => "void".into(),
            ReturnType::Type(_, typ) => {
                let ty = ts_type(typ.deref());
                if ty == "Promise" {
                    "void".to_string()
                } else {
                    ty
                }
            }
        };

        let mut args_decl = Vec::new();
        if args.len() > 0 {
            args_decl.push(format!("args: {{ {} }}", args.join(", ")));
        };
        if method.is_mut() {
            args_decl.push("gas?: any".into());
        }
        if method.is_payable() {
            args_decl.push("amount?: any".into());
        }

        format!(
            "{}({}): Promise<{}>;",
            method.sig.ident,
            args_decl.join(", "),
            ret_type
        )
    }
}

#[cfg(test)]
mod tests {

    use crate::ts::ts_type;

    #[test]
    #[should_panic(expected = "Option used with no generic arg")]
    fn ts_type_on_option_with_no_args_should_panic() {
        ts_type(&syn::parse_str("Option").unwrap());
    }

    #[test]
    #[should_panic(expected = "Option expects 1 generic(s) argument(s), found 2")]
    fn ts_type_on_option_with_more_than_one_arg_should_panic() {
        ts_type(&syn::parse_str("Option<String, U128>").unwrap());
    }

    #[test]
    #[should_panic(expected = "Vec used with no generic arg")]
    fn ts_type_on_vec_with_no_args_should_panic() {
        ts_type(&syn::parse_str("Vec").unwrap());
    }

    #[test]
    #[should_panic(expected = "Vec expects 1 generic(s) argument(s), found 3")]
    fn ts_type_on_vec_with_more_than_one_arg_should_panic() {
        ts_type(&syn::parse_str("Vec<String, U128, u32>").unwrap());
    }

    #[test]
    #[should_panic(expected = "HashMap used with no generic arguments")]
    fn ts_type_on_hashmap_with_no_args_should_panic() {
        ts_type(&syn::parse_str("HashMap").unwrap());
    }

    #[test]
    #[should_panic(expected = "HashMap expects 2 generic(s) argument(s), found 1")]
    fn ts_type_on_hashmap_with_less_than_two_args_should_panic() {
        ts_type(&syn::parse_str("HashMap<U64>").unwrap());
    }
}
