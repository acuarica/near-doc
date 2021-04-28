#![deny(warnings)]

use clap::Clap;
use near_syn::{parse_rust, ts::TS, Args};
use std::env;

Args!(env!("CARGO_BIN_NAME"));

fn main() {
    let mut args = Args::parse();
    let mut ts = TS::new(std::io::stdout());
    ts.ts_prelude(args.now(), env!("CARGO_BIN_NAME"));

    for file_name in args.files {
        let ast = parse_rust(file_name);
        ts.ts_items(&ast.items);
    }
    ts.ts_main_type();
    ts.ts_contract_methods();
}
