use near_syn::ts::TS;
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

    let mut ts = TS::new(Vec::new());
    ts.ts_items(&ast.items);
    let out = String::from_utf8(ts.buf).unwrap();
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
