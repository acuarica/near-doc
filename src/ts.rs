use syn::PathArguments;

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
/// Also nested types are converted to TypeScript.
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
/// Incorrect type arguments in standard library generics types panics.
///
/// ```should_panic
/// near_syn::ts::ts_type(&syn::parse_str("Option").unwrap());
/// ```
/// ```should_panic
/// near_syn::ts::ts_type(&syn::parse_str("Option<String, String>").unwrap());
/// ```
/// ```should_panic
/// near_syn::ts::ts_type(&syn::parse_str("HashMap<U64>").unwrap());
/// ```
pub fn ts_type(ty: &syn::Type) -> String {
    #[derive(PartialEq, PartialOrd)]
    enum Assoc {
        Single,
        Brackets,
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
    fn ts_type_assoc(ty: &syn::Type) -> (String, Assoc) {
        match ty {
            syn::Type::Path(p) => match crate::join_path(&p.path).as_str() {
                "bool" => single("boolean"),
                "u32" => single("number"),
                "u64" => single("number"),
                "String" => single("string"),
                "Option" => {
                    if let PathArguments::AngleBracketed(args) = &p.path.segments[0].arguments {
                        if args.args.len() != 1 {
                            panic!("incorrect Option usage");
                        }
                        if let syn::GenericArgument::Type(t) = &args.args[0] {
                            let ta = ts_type_assoc(t);
                            return (format!("{}|null", use_paren(ta, Assoc::Or)), Assoc::Or);
                        }
                    }
                    panic!("not expected");
                }
                "Vec" => {
                    if let PathArguments::AngleBracketed(args) = &p.path.segments[0].arguments {
                        if let syn::GenericArgument::Type(t) = &args.args[0] {
                            let ta = ts_type_assoc(&t);
                            return (
                                format!("{}[]", use_paren(ta, Assoc::Brackets)),
                                Assoc::Brackets,
                            );
                        }
                    }
                    panic!("not expected");
                }
                "HashMap" => {
                    if let PathArguments::AngleBracketed(args) = &p.path.segments[0].arguments {
                        if let syn::GenericArgument::Type(tk) = &args.args[0] {
                            if let syn::GenericArgument::Type(tv) = &args.args[1] {
                                let (tks, _) = ts_type_assoc(&tk);
                                let (tvs, _) = ts_type_assoc(&tv);
                                return (format!("Record<{}, {}>", tks, tvs), Assoc::Single);
                            }
                        }
                    }
                    panic!("not expected");
                }
                s => single(s),
            },
            _ => panic!("type not supported"),
        }
    }
    ts_type_assoc(ty).0
}
