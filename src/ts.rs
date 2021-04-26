//! Functions to transpile Rust to TypeScript.

use crate::{join_path, near_syn::NearMethod, write_docs, NearImpl, NearSerde};
use std::ops::Deref;
use syn::{
    Attribute, File, ImplItem, ImplItemMethod, Item, ItemEnum, ItemImpl, ItemStruct, PathArguments,
    ReturnType, Type,
};

/// asdf
pub struct TS<T> {
    name: String,
    impl_count: u32,
    interfaces: Vec<String>,
    view_methods: Vec<String>,
    change_methods: Vec<String>,
    ///
    pub buf: T,
}

macro_rules! ln {
    ($this:ident, $($arg:tt)*) => ({
        writeln!($this.buf, $($arg)*).unwrap()
    })
}

impl<T: std::io::Write> TS<T> {
    /// ```
    /// use near_syn::ts::TS;
    ///
    /// let mut ts = TS::new(Vec::new());
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

    /// ```
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

    /// ```
    /// use near_syn::ts::TS;
    ///
    /// let mut ts = TS::new(Vec::new());
    /// ts.ts_main_type();
    /// assert_eq!(String::from_utf8_lossy(&ts.buf), "");
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

    /// asfd
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

    /// asdf
    pub fn ts_unit(&mut self, ast: &File) {
        for item in &ast.items {
            match item {
                Item::Enum(item_enum) => {
                    self.ts_enum(&item_enum);
                }
                Item::Impl(impl_item) => {
                    if impl_item.is_bindgen() && impl_item.has_exported_methods() {
                        self.ts_doc(&impl_item.attrs, "");
                        if let Some((_, trait_path, _)) = &impl_item.trait_ {
                            let trait_name = join_path(trait_path);
                            self.interfaces.push(trait_name.clone());
                            ln!(self, "export interface {} {{", trait_name);
                        } else {
                            if let syn::Type::Path(type_path) = &*impl_item.self_ty {
                                let impl_name = format!("Self{}", self.impl_count);
                                self.impl_count += 1;
                                self.name = join_path(&type_path.path);
                                self.interfaces.push(impl_name.clone());
                                ln!(self, "export interface {} {{", impl_name);
                            } else {
                                panic!("name not found")
                            }
                        }

                        self.ts_methods(&impl_item);
                        ln!(self, "}}\n");
                    }
                }
                Item::Type(item_type) => self.ts_typedef(&item_type),
                Item::Struct(item_struct) => {
                    if item_struct.is_serde() {
                        self.ts_struct(&item_struct)
                    }
                }
                _ => {}
            }
        }
    }

    fn ts_typedef(&mut self, item_type: &syn::ItemType) {
        self.ts_doc(&item_type.attrs, "");
        ln!(
            self,
            "export type {} = {};",
            item_type.ident,
            ts_type(&item_type.ty)
        );
        ln!(self, "");
    }

    fn ts_struct(&mut self, item_struct: &ItemStruct) {
        self.ts_doc(&item_struct.attrs, "");
        match &item_struct.fields {
            syn::Fields::Named(fields) => {
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
            syn::Fields::Unnamed(fields) => {
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
            syn::Fields::Unit => panic!("unit struct no supported"),
        }
    }

    fn ts_enum(&mut self, item_enum: &ItemEnum) {
        if item_enum.is_serde() {
            self.ts_doc(&item_enum.attrs, "");
            ln!(self, "export enum {} {{", item_enum.ident);
            for variant in &item_enum.variants {
                ln!(self, "    {},", variant.ident);
            }
            ln!(self, "}}\n");
        }
    }

    fn ts_methods(&mut self, input: &ItemImpl) {
        for impl_item in input.items.iter() {
            if let ImplItem::Method(method) = impl_item {
                if method.is_exported(input) {
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
