mod ts_items {
    use near_syn::{contract::Contract, ts::ts_items};
    use quote::quote;
    use syn::{parse2, File};

    #[test]
    fn it_should_omit_non_near_sdk_items() {
        let ast: syn::File = parse2(quote! {
            mod empty_mod { }
            trait A { }
        })
        .unwrap();
        let mut contract = Contract::new();
        contract.push_ast(ast);
        let mut buf = Vec::new();
        ts_items(&mut buf, &mut contract).unwrap();
        assert_eq!(String::from_utf8_lossy(&buf), "");
    }

    #[test]
    fn it_should_forward_trait_comments() {
        let ast: File = parse2(quote! {

            /// doc for IContract
            trait IContract {
                /// doc for IContract::get
                fn get(&self, f128: U128) -> U128;
            }

            #[near_bindgen]
            impl IContract for Contract {
                pub fn get(&self, f128: U128) -> U128 {
                    f128
                }
            }

        })
        .unwrap();

        let mut buf = Vec::new();
        let mut contract = Contract::new();
        contract.push_ast(ast);
        ts_items(&mut buf, &contract).unwrap();
        let out = String::from_utf8(buf).unwrap();
        assert_eq!(
            out,
            r#"/**
 * doc for IContract
 */
export interface IContract {
    /**
     * doc for IContract::get
     */
    get(args: { f128: U128 }): Promise<U128>;

}

"#
        );
    }

    #[test]
    fn it_should_merge_trait_and_impl_comments() {
        let ast: File = parse2(quote! {

            /// doc for IContract
            trait IContract {
                /// doc for IContract::get
                fn get(&self, f128: U128) -> U128;
            }

            /// doc in Contract
            #[near_bindgen]
            impl IContract for Contract {
                /// doc in Contract::get
                pub fn get(&self, f128: U128) -> U128 {
                    f128
                }
            }

        })
        .unwrap();

        let mut buf = Vec::new();
        let mut contract = Contract::new();
        contract.push_ast(ast);
        ts_items(&mut buf, &contract).unwrap();
        let out = String::from_utf8(buf).unwrap();
        assert_eq!(
            out,
            r#"/**
 * doc in Contract
 * doc for IContract
 */
export interface IContract {
    /**
     * doc in Contract::get
     * doc for IContract::get
     */
    get(args: { f128: U128 }): Promise<U128>;

}

"#
        );
    }
}

mod ts_impl {

    use near_syn::{contract::Contract, ts::ts_impl};
    use quote::quote;
    use syn::parse2;

    #[test]
    fn it_should_omit_non_bindgen_impls() {
        let item_impl = &parse2(quote! {
            #[near_bindgen]
            impl A { }
        })
        .unwrap();
        let mut buf = Vec::new();
        ts_impl(&mut buf, item_impl, &Contract::new()).unwrap();
        assert_eq!(String::from_utf8_lossy(&buf), "");

        let item_impl = &parse2(quote! {
            #[near_bindgen]
            impl A {
                #[private]
                pub fn set(&mut self) {}
             }
        })
        .unwrap();
        let mut buf = Vec::new();
        ts_impl(&mut buf, item_impl, &Contract::new()).unwrap();
        assert_eq!(String::from_utf8_lossy(&buf), "");

        let item_impl = &parse2(quote! {
            impl A {
                pub fn set(&mut self) {}
            }
        })
        .unwrap();
        let mut buf = Vec::new();
        ts_impl(&mut buf, item_impl, &Contract::new()).unwrap();
        assert_eq!(String::from_utf8_lossy(&buf), "");
    }

    #[test]
    #[should_panic = "Impl struct name not supported"]
    fn it_should_panic_() {
        let item_impl = &parse2(quote! {
            #[near_bindgen]
            impl *const u32 {
                pub fn set(&mut self) {}
            }
        })
        .unwrap();
        let mut buf = Vec::new();
        ts_impl(&mut buf, item_impl, &Contract::new()).unwrap();
    }
}

mod ts_struct {

    use near_syn::ts::ts_struct;
    use quote::quote;
    use syn::parse2;

    #[test]
    fn it_should_omit_non_serde_structs() {
        let mut buf = Vec::new();
        let item_struct = &parse2(quote! {
            struct A { }
        })
        .unwrap();
        ts_struct(&mut buf, item_struct).unwrap();
        assert_eq!(String::from_utf8_lossy(&buf), "");
    }

    #[test]
    #[should_panic(expected = "unit struct no supported")]
    fn it_should_panic_when_unit_struct_is_provided() {
        let mut buf = Vec::new();
        let item_struct = &parse2(quote! {
            #[derive(Serialize)]
            struct A;
        })
        .unwrap();
        ts_struct(&mut buf, item_struct).unwrap();
        assert_eq!(String::from_utf8_lossy(&buf), "");
    }
}

mod ts_enum {
    use near_syn::ts::ts_enum;
    use quote::quote;
    use syn::parse2;

    #[test]
    fn it_should_omit_non_serde_enums() {
        let mut buf = Vec::new();
        let item_enum = &parse2(quote! {
            enum E {
                V1,
            }
        })
        .unwrap();
        ts_enum(&mut buf, item_enum).unwrap();
        assert_eq!(String::from_utf8_lossy(&buf), "");
    }
}

mod ts_type {

    use near_syn::ts::ts_type;
    use syn::parse_str;

    #[test]
    fn it_should_convert_rust_primitive_types() {
        assert_eq!(ts_type(&parse_str("bool").unwrap()), "boolean");
        assert_eq!(ts_type(&parse_str("i8").unwrap()), "number");
        assert_eq!(ts_type(&parse_str("u8").unwrap()), "number");
        assert_eq!(ts_type(&parse_str("i16").unwrap()), "number");
        assert_eq!(ts_type(&parse_str("u16").unwrap()), "number");
        assert_eq!(ts_type(&parse_str("i32").unwrap()), "number");
        assert_eq!(ts_type(&parse_str("u32").unwrap()), "number");
        assert_eq!(ts_type(&parse_str("u64").unwrap()), "number");
        assert_eq!(ts_type(&parse_str("i64").unwrap()), "number");
        assert_eq!(ts_type(&parse_str("String").unwrap()), "string");
    }

    #[test]
    fn it_should_convert_rust_unit_type() {
        assert_eq!(ts_type(&parse_str("()").unwrap()), "void");
    }

    #[test]
    fn it_should_convert_rust_shared_reference_types() {
        assert_eq!(ts_type(&parse_str("&String").unwrap()), "string");
        assert_eq!(ts_type(&parse_str("&bool").unwrap()), "boolean");
        assert_eq!(ts_type(&parse_str("&u32").unwrap()), "number");
        assert_eq!(ts_type(&parse_str("&TokenId").unwrap()), "TokenId");
    }

    #[test]
    fn it_should_convert_rust_standard_and_collection_types() {
        assert_eq!(ts_type(&parse_str("Option<U64>").unwrap()), "U64|null");
        assert_eq!(
            ts_type(&parse_str("Option<String>").unwrap()),
            "string|null"
        );
        assert_eq!(
            ts_type(&parse_str("Vec<ValidAccountId>").unwrap()),
            "ValidAccountId[]"
        );
        assert_eq!(
            ts_type(&parse_str("HashSet<ValidAccountId>").unwrap()),
            "ValidAccountId[]"
        );
        assert_eq!(
            ts_type(&parse_str("BTreeSet<ValidAccountId>").unwrap()),
            "ValidAccountId[]"
        );
        assert_eq!(
            ts_type(&parse_str("HashMap<AccountId, U128>").unwrap()),
            "Record<AccountId, U128>"
        );
        assert_eq!(
            ts_type(&parse_str("BTreeMap<AccountId, U128>").unwrap()),
            "Record<AccountId, U128>"
        );
    }

    #[test]
    fn it_should_convert_rust_nested_types() {
        assert_eq!(
            ts_type(&parse_str("HashMap<AccountId, Vec<U128>>").unwrap()),
            "Record<AccountId, U128[]>"
        );
        assert_eq!(
            ts_type(&parse_str("Vec<Option<U128>>").unwrap()),
            "(U128|null)[]"
        );
        assert_eq!(
            ts_type(&parse_str("Option<Vec<U128>>").unwrap()),
            "U128[]|null"
        );
        assert_eq!(
            ts_type(&parse_str("Option<Option<U64>>").unwrap()),
            "U64|null|null"
        );
        assert_eq!(ts_type(&parse_str("Vec<Vec<U64>>").unwrap()), "U64[][]");
        assert_eq!(ts_type(&parse_str("(U64)").unwrap()), "U64");
    }

    #[test]
    fn it_should_convert_rust_tuple_types() {
        assert_eq!(
            ts_type(&parse_str("(U64, String, Vec<u32>)").unwrap()),
            "[U64, string, number[]]"
        );
    }

    #[test]
    #[ignore = "Path not supported"]
    fn it_should_convert_rust_path_types() {
        assert_eq!(ts_type(&parse_str("std::vec::Vec<U64>").unwrap()), "U64[]");
    }

    #[test]
    #[should_panic(expected = "Option used with no generic arg")]
    fn it_should_panic_on_option_with_no_args() {
        ts_type(&syn::parse_str("Option").unwrap());
    }

    #[test]
    #[should_panic(expected = "Option expects 1 generic(s) argument(s), found 2")]
    fn it_should_panic_on_option_with_more_than_one_arg() {
        ts_type(&syn::parse_str("Option<String, U128>").unwrap());
    }

    #[test]
    #[should_panic(expected = "No type provided for Option")]
    fn it_should_panic_on_option_with_no_generic_type_argument() {
        ts_type(&syn::parse_str("Option<'a>").unwrap());
    }

    #[test]
    #[should_panic(expected = "Vec used with no generic arg")]
    fn it_should_panic_on_vec_with_no_args() {
        ts_type(&syn::parse_str("Vec").unwrap());
    }

    #[test]
    #[should_panic(expected = "Vec expects 1 generic(s) argument(s), found 3")]
    fn it_should_panic_on_vec_with_more_than_one_arg() {
        ts_type(&syn::parse_str("Vec<String, U128, u32>").unwrap());
    }

    #[test]
    #[should_panic(expected = "HashMap used with no generic arguments")]
    fn it_should_panic_on_hashmap_with_no_args() {
        ts_type(&syn::parse_str("HashMap").unwrap());
    }

    #[test]
    #[should_panic(expected = "HashMap expects 2 generic(s) argument(s), found 1")]
    fn it_should_panic_on_hashmap_with_less_than_two_args() {
        ts_type(&syn::parse_str("HashMap<U64>").unwrap());
    }

    #[test]
    #[should_panic(expected = "type not supported: ")]
    fn it_should_panic_with_not_supported_type() {
        ts_type(&syn::parse_str("*const u32").unwrap());
    }
}

mod ts_ret_type {

    use near_syn::ts::ts_ret_type;
    use syn::parse_str;

    #[test]
    fn it_should_treat_default_return_type_as_unit() {
        assert_eq!(ts_ret_type(&parse_str(" ").unwrap()), "void");
    }

    #[test]
    fn it_should_convert_primitive_and_complex_return_types() {
        assert_eq!(ts_ret_type(&parse_str("-> Vec<Token>").unwrap()), "Token[]");
        assert_eq!(ts_ret_type(&parse_str("-> u32").unwrap()), "number");
    }

    #[test]
    fn it_should_convert_promises_to_void() {
        assert_eq!(ts_ret_type(&parse_str("-> Promise<u32>").unwrap()), "void");
        assert_eq!(
            ts_ret_type(&parse_str("-> PromiseOrValue<String>").unwrap()),
            "void"
        );
    }
}

mod ts_doc {
    use std::io;

    use near_syn::ts::ts_doc;
    use quote::quote;
    use syn::parse2;

    #[test]
    fn it_should_emit_empty_doc_comments() -> io::Result<()> {
        let mut buf = Vec::new();
        let item = parse2::<syn::ItemType>(quote! {
            type T = u64;
        });

        let result = r#"/**
 */
"#;
        ts_doc(&mut buf, &item.unwrap().attrs, "")?;
        assert_eq!(String::from_utf8_lossy(&buf), result);

        Ok(())
    }

    #[test]
    fn it_should_emit_single_line_doc_comments() -> io::Result<()> {
        let mut buf = Vec::new();
        let item = parse2::<syn::ItemType>(quote! {
            /// Doc-comments are translated.
            type T = u64;
        });
        let result = r#"/**
 * Doc-comments are translated.
 */
"#;
        ts_doc(&mut buf, &item.unwrap().attrs, "")?;
        assert_eq!(String::from_utf8_lossy(&buf), result);

        Ok(())
    }

    #[test]
    fn it_should_emit_multiple_lines_doc_comments() -> io::Result<()> {
        let mut buf = Vec::new();
        let item = parse2::<syn::ItemType>(quote! {
            /// Doc-comments are translated.
            /// Multi-line is also translated.
            ///
            ///Adds a leading space.
            ///    Trims leading spaces.
            type T = u64;
        });

        let result = r#"/**
 * Doc-comments are translated.
 * Multi-line is also translated.
 * 
 * Adds a leading space.
 * Trims leading spaces.
 */
"#;
        ts_doc(&mut buf, &item.unwrap().attrs, "")?;
        assert_eq!(String::from_utf8_lossy(&buf), result);

        Ok(())
    }
}
