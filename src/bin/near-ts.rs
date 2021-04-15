#![deny(warnings)]

use chrono::Utc;
use clap::Clap;
use near_syn::{
    derives, has_attr, is_init, is_mut, join_path, parse_rust,
    ts::{ts_sig, ts_type},
    write_docs, Args, NearMethod,
};
use std::env;
use syn::{
    Attribute, File, ImplItem,
    Item::{Enum, Impl, Struct, Type},
    ItemImpl, ItemStruct,
};

Args!(env!("CARGO_BIN_NAME"));

fn main() {
    let mut args = Args::parse();
    let mut ts = TS::new(std::io::stdout());
    ts.ts_prelude(args.now());

    for file_name in args.files {
        let ast = parse_rust(file_name);
        ts.ts_unit(&ast);
    }
    ts.ts_main_type();
    ts.ts_contract_methods();
}

struct TS<T> {
    name: String,
    interfaces: Vec<String>,
    view_methods: Vec<String>,
    change_methods: Vec<String>,
    file: T,
}

macro_rules! ln {
    ($this:ident, $($arg:tt)*) => ({
        writeln!($this.file, $($arg)*).unwrap()
    })
}

impl<T: std::io::Write> TS<T> {
    fn new(file: T) -> Self {
        Self {
            name: String::new(),
            interfaces: Vec::new(),
            view_methods: Vec::new(),
            change_methods: Vec::new(),
            file,
        }
    }

    fn ts_prelude(&mut self, now: String) {
        ln!(
            self,
            "// TypeScript bindings generated with {} v{} {} on {}\n",
            env!("CARGO_BIN_NAME"),
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

    fn ts_main_type(&mut self) {
        if !self.name.is_empty() && !self.interfaces.is_empty() {
            ln!(
                self,
                "export type {} = {};\n",
                self.name,
                self.interfaces.join(" & ")
            );
        }
    }

    fn ts_contract_methods(&mut self) {
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

    fn ts_unit(&mut self, ast: &File) {
        for item in &ast.items {
            match item {
                Enum(_) => {}
                Impl(impl_item) => {
                    if has_attr(&impl_item.attrs, "near_bindgen") {
                        if let Some((_, trait_path, _)) = &impl_item.trait_ {
                            let trait_name = join_path(trait_path);
                            self.interfaces.push(trait_name.clone());
                            ln!(self, "export interface {} {{", trait_name);
                        } else {
                            if let syn::Type::Path(type_path) = &*impl_item.self_ty {
                                self.name = join_path(&type_path.path);
                                self.interfaces.push("Self".to_string());
                                ln!(self, "export interface Self {{");
                            } else {
                                panic!("name not found")
                            }
                        }

                        self.ts_methods(&impl_item);
                        ln!(self, "}}\n");
                    }
                }
                Type(item_type) => self.ts_typedef(&item_type),
                Struct(item_struct) => {
                    if derives(&item_struct.attrs, "Serialize")
                        || derives(&item_struct.attrs, "Deserialize")
                    {
                        self.ts_struct(&item_struct)
                    }
                }
                _ => {}
            }
        }
    }

    fn ts_typedef(&mut self, item_type: &syn::ItemType) {
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
        ln!(self, "export interface {} {{", item_struct.ident);
        for field in &item_struct.fields {
            if let Some(field_name) = &field.ident {
                let ty = ts_type(&field.ty);
                self.ts_doc(&field.attrs, "    ");
                ln!(self, "    {}: {};\n", field_name, ty);
            } else {
                panic!("tuple struct no supported");
            }
        }
        ln!(self, "}}");
        ln!(self, "");
    }

    fn ts_methods(&mut self, input: &ItemImpl) {
        for impl_item in input.items.iter() {
            if let ImplItem::Method(method) = impl_item {
                if method.is_exported(input) {
                    if !is_init(&method) {
                        if is_mut(&method) {
                            self.change_methods.push(method.sig.ident.to_string());
                        } else {
                            self.view_methods.push(method.sig.ident.to_string());
                        }
                    }
                    self.ts_doc(&method.attrs, "    ");
                    ln!(self, "    {}\n", ts_sig(&method));
                }
            }
        }
    }

    fn ts_doc(&mut self, attrs: &Vec<Attribute>, indent: &str) {
        ln!(self, "{}/**", indent);
        write_docs(&mut self.file, attrs, |l| format!("{} * {}", indent, l));
        ln!(self, "{} */", indent);
    }
}
