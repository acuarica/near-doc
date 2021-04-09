// #![deny(warnings)]

use chrono::Utc;
use near_doc::{derives, has_attr, is_mut, is_public, join_path, parse_rust};
use std::{env, ops::Deref};
use syn::{
    File, ImplItem, ImplItemMethod,
    Item::{Enum, Impl, Struct, Type},
    ItemImpl, ItemStruct, PathArguments,
};

fn main() {
    let mut args = env::args();
    args.next();

    println!(
        "// TypeScript bindings generated with {} v{} {} on {}\n",
        env!("CARGO_BIN_NAME"),
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_REPOSITORY"),
        Utc::now()
    );

    println!("// Exports common NEAR Rust SDK types");
    println!("export type U64 = string;");
    println!("export type U128 = string;");
    println!("export type AccountId = string;");
    println!("export type ValidAccountId = string;");
    println!("");

    let mut ts = TS::new();
    for file_name in args {
        let ast = parse_rust(file_name);
        ts.ts_unit(&ast);
    }
    ts.ts_main_type();
    ts.ts_contract_methods();
}

struct TS {
    name: String,
    interfaces: Vec<String>,
    view_methods: Vec<String>,
    change_methods: Vec<String>,
}

impl TS {
    fn new() -> Self {
        Self {
            name: String::new(),
            interfaces: Vec::new(),
            view_methods: Vec::new(),
            change_methods: Vec::new(),
        }
    }

    fn ts_main_type(&self) {
        println!(
            "export type {} = {};\n",
            self.name,
            self.interfaces.join(" & ")
        );
    }

    fn ts_contract_methods(&self) {
        fn pack(methods: &Vec<String>) -> String {
            methods
                .iter()
                .map(|m| format!("        {:?},\n", m))
                .collect::<Vec<String>>()
                .join("")
        }

        println!("export const {}Methods = {{", self.name);
        println!("    viewMethods: [\n{}    ],", pack(&self.view_methods));
        println!("    changeMethods: [\n{}    ],", pack(&self.change_methods));
        println!("}};");
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
                            println!("export interface {} {{", trait_name);
                        } else {
                            if let syn::Type::Path(type_path) = &*impl_item.self_ty {
                                self.name = join_path(&type_path.path);
                                self.interfaces.push("Self".to_string());
                                println!("export interface Self {{");
                            } else {
                                panic!("name not found")
                            }
                        }

                        self.ts_methods(&impl_item);
                        println!("}}\n");
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

    fn ts_typedef(&self, item_type: &syn::ItemType) {
        println!(
            "export type {} = {};",
            item_type.ident,
            self.ts_type(&item_type.ty)
        );
        println!("");
    }

    fn ts_struct(&self, item_struct: &ItemStruct) {
        println!("export interface {} {{", item_struct.ident);
        for field in &item_struct.fields {
            if let Some(field_name) = &field.ident {
                let ty = self.ts_type(&field.ty);
                println!("    {}: {};", field_name, ty);
            } else {
                panic!("tuple struct no supported");
            }
        }
        println!("}}");
        println!("");
    }

    fn ts_type(&self, ty: &syn::Type) -> String {
        match ty {
            syn::Type::Path(p) => match join_path(&p.path).as_str() {
                "u32" => "number".to_string(),
                "u64" => "number".to_string(),
                "String" => "string".to_string(),
                "Option" => {
                    if let PathArguments::AngleBracketed(args) = &p.path.segments[0].arguments {
                        if let syn::GenericArgument::Type(t) = &args.args[0] {
                            return format!("{}|null", self.ts_type(&t));
                        }
                    }
                    panic!("not expected");
                }
                "Vec" => {
                    if let PathArguments::AngleBracketed(args) = &p.path.segments[0].arguments {
                        if let syn::GenericArgument::Type(t) = &args.args[0] {
                            return format!("{}[]", self.ts_type(&t));
                        }
                    }
                    panic!("not expected");
                }
                "HashMap" => {
                    if let PathArguments::AngleBracketed(args) = &p.path.segments[0].arguments {
                        if let syn::GenericArgument::Type(tk) = &args.args[0] {
                            if let syn::GenericArgument::Type(tv) = &args.args[1] {
                                return format!(
                                    "Record<{}, {}>",
                                    self.ts_type(&tk),
                                    self.ts_type(tv)
                                );
                            }
                        }
                    }
                    panic!("not expected");
                }

                s => s.to_string(),
            },
            _ => panic!("type not supported"),
        }
    }

    fn ts_methods(&mut self, input: &ItemImpl) {
        for impl_item in input.items.iter() {
            if let ImplItem::Method(method) = impl_item {
                if is_public(method) || input.trait_.is_some() {
                    if is_mut(&method) || has_attr(&method.attrs, "init") {
                        self.change_methods.push(method.sig.ident.to_string());
                    } else {
                        self.view_methods.push(method.sig.ident.to_string());
                    }
                    let sig = self.extract_sig(&method);
                    println!("    {}\n", sig);
                }
            }
        }
    }

    fn extract_sig(&self, method: &ImplItemMethod) -> String {
        let mut args = Vec::new();
        for arg in method.sig.inputs.iter() {
            match arg {
                syn::FnArg::Typed(pat_type) => {
                    if let syn::Pat::Ident(pat_ident) = pat_type.pat.deref() {
                        let type_name = if let syn::Type::Path(_type_path) = &*pat_type.ty {
                            self.ts_type(&pat_type.ty)
                        // join_path(&type_path.path)
                        } else {
                            panic!("not support sig type");
                        };
                        let arg_ident = &pat_ident.ident;
                        args.push(format!("{}: {}", arg_ident, type_name));
                    }
                }
                _ => {}
            }
        }

        let ret_type = match &method.sig.output {
            syn::ReturnType::Default => "void".to_string(),
            syn::ReturnType::Type(_, typ) => {
                self.ts_type(typ.deref())
                // let typ = typ.deref();
                // let type_name = proc_macro2::TokenStream::from(quote! { #typ }).to_string();
                // if type_name == "Self" {
                //     "void".to_string()
                // } else {
                //     type_name
                // }
            }
        };

        let args_decl = if args.len() == 0 {
            "".to_string()
        } else {
            format!("args: {{ {} }}", args.join(", "))
        };
        format!(
            "{}({}): Promise<{}>;",
            method.sig.ident, args_decl, ret_type
        )
    }
}
