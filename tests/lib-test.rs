use near_syn::ts::ts_type;

#[test]
#[should_panic(expected = "Option used with no generic arg")]
fn ts_type_on_option_with_no_args_should_panic() {
    ts_type(&syn::parse_str("Option").unwrap());
}
