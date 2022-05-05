use near_syn::{contract::Contract, ts::ts_items};
use quote::quote;
use syn::{parse2, File};

#[test]
fn ts_should_forward_trait_comments() {
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
    contract.forward_traits(&ast.items);
    ts_items(&mut buf, &ast.items, &mut contract);
    let out = String::from_utf8(buf).unwrap();
    assert_eq!(
        out,
        r#"/**
 *  doc for IContract
 */
export interface IContract {
    /**
     *  doc for IContract::get
     */
    get(args: { f128: U128 }): Promise<U128>;

}

"#
    );
}

#[test]
fn ts_should_merge_trait_and_impl_comments() {
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
    contract.forward_traits(&ast.items);
    ts_items(&mut buf, &ast.items, &mut contract);
    let out = String::from_utf8(buf).unwrap();
    assert_eq!(
        out,
        r#"/**
 *  doc in Contract
 *  doc for IContract
 */
export interface IContract {
    /**
     *  doc in Contract::get
     *  doc for IContract::get
     */
    get(args: { f128: U128 }): Promise<U128>;

}

"#
    );
}
