//! Functions to transpile Rust to TypeScript.

use std::ops::Deref;

use syn::{ImplItemMethod, PathArguments, ReturnType};

/// Return the TypeScript equivalent type of the Rust type represented by `ty`.
/// Rust primitives types are included.
///
/// ```
/// use syn::parse_str;
/// use near_syn::ts::ts_type;
///
/// assert_eq!(ts_type(&parse_str("bool").unwrap()), "boolean");
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
/// assert_eq!(ts_type(&parse_str("HashMap<AccountId, U128>").unwrap()), "Record<AccountId, U128>");
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
/// ```
///
/// ## Panics
///
/// Panics when standard library generics types are used incorrectly.
/// For example `Option` or `HashMap<U64>`.
/// This situation can only happen on Rust source files that were **not** type-checked by `rustc`.
pub fn ts_type(ty: &syn::Type) -> String {
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
    fn gen_args<'a>(p: &'a syn::TypePath, nargs: usize, name: &str) -> Vec<&'a syn::Type> {
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

    fn ts_type_assoc(ty: &syn::Type) -> (String, Assoc) {
        match ty {
            syn::Type::Path(p) => match crate::join_path(&p.path).as_str() {
                "bool" => single("boolean"),
                "u32" => single("number"),
                "u64" => single("number"),
                "String" => single("string"),
                "Option" => {
                    let targs = gen_args(p, 1, "Option");
                    let ta = ts_type_assoc(&targs[0]);
                    return (format!("{}|null", use_paren(ta, Assoc::Or)), Assoc::Or);
                }
                "Vec" => {
                    let targs = gen_args(p, 1, "Vec");
                    let ta = ts_type_assoc(&targs[0]);
                    return (format!("{}[]", use_paren(ta, Assoc::Vec)), Assoc::Vec);
                }
                "HashMap" => {
                    let targs = gen_args(p, 2, "HashMap");
                    let (tks, _) = ts_type_assoc(&targs[0]);
                    let (tvs, _) = ts_type_assoc(&targs[1]);
                    return (format!("Record<{}, {}>", tks, tvs), Assoc::Single);
                }
                s => single(s),
            },
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
/// ```
pub fn ts_sig(method: &ImplItemMethod) -> String {
    let mut args = Vec::new();
    for arg in method.sig.inputs.iter() {
        match arg {
            syn::FnArg::Typed(pat_type) => {
                if let syn::Pat::Ident(pat_ident) = pat_type.pat.deref() {
                    let type_name = if let syn::Type::Path(_type_path) = &*pat_type.ty {
                        ts_type(&pat_type.ty)
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

    if crate::is_init(&method) {
        format!("{}: {{ {} }};", method.sig.ident, args.join(", "),)
    } else {
        let ret_type = match &method.sig.output {
            ReturnType::Default => "void".into(),
            ReturnType::Type(_, typ) => ts_type(typ.deref()),
        };

        let mut args_decl = Vec::new();
        if args.len() > 0 {
            args_decl.push(format!("args: {{ {} }}", args.join(", ")));
        };
        if crate::is_mut(&method) {
            args_decl.push("gas?: any".into());
        }
        if crate::is_payable(&method) {
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
