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

use std::io;

use near_syn::{contract::Contract, ts::ts_items};
use quote::quote;
use syn::{parse2, File};

#[test]
fn ts_should_forward_trait_comments() -> io::Result<()> {
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
    ts_items(&mut buf, &ast.items, &mut contract)?;
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

    Ok(())
}

#[test]
fn ts_should_merge_trait_and_impl_comments() -> io::Result<()> {
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
    ts_items(&mut buf, &ast.items, &mut contract)?;
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

    Ok(())
}
