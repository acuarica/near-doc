mod near_impl {

    use near_syn::NearImpl;
    use proc_macro2::TokenStream;
    use quote::quote;
    use syn::{parse2, ItemImpl};

    fn parse_item_impl(tokens: TokenStream) -> ItemImpl {
        let item_impl = parse2(tokens).unwrap();
        item_impl
    }

    #[test]
    fn it_should_return_true_when_bindgen_is_present() {
        let item_impl = parse_item_impl(quote! {
            #[not_near_bindgen]
            impl Contract {
                pub fn get(&self, f128: U128) -> U128 { f128 }
            }
        });
        assert!(!item_impl.is_bindgen());

        let item_impl = parse_item_impl(quote! {
            #[near_bindgen]
            impl Contract {
                pub fn get(&self, f128: U128) -> U128 { f128 }
            }
        });
        assert!(item_impl.is_bindgen());

        let item_impl = parse_item_impl(quote! {
            #[near_bindgen]
            impl IContract for Contract {
                pub fn get(&self, f128: U128) -> U128 { f128 }
            }
        });
        assert!(item_impl.is_bindgen());
    }

    #[test]
    fn it_should_return_trait_name_if_present() {
        let item_impl = parse_item_impl(quote! {
            impl Contract { }
        });
        assert_eq!(item_impl.get_trait_name(), None);

        let item_impl = parse_item_impl(quote! {
            impl IContract for Contract { }
        });
        assert_eq!(item_impl.get_trait_name(), Some("IContract".to_string()));
    }

    #[test]
    fn it_should_return_impl_name() {
        let item_impl = parse_item_impl(quote! {
            impl Contract { }
        });
        assert_eq!(item_impl.get_impl_name(), Some("Contract".to_string()));

        let item_impl = parse_item_impl(quote! {
            impl IContract for Contract { }
        });
        assert_eq!(item_impl.get_impl_name(), Some("Contract".to_string()));

        let item_impl = parse_item_impl(quote! {
            impl IContract for (Contract) { }
        });
        assert_eq!(item_impl.get_impl_name(), None);

        let item_impl = parse_item_impl(quote! {
            impl IContract for (Contract, Contract) { }
        });
        assert_eq!(item_impl.get_impl_name(), None);
    }
}
