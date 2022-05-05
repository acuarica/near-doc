#![deny(warnings)]

use chrono::Utc;
use clap::Parser;
use near_syn::{
    contract::Contract,
    md::{md_footer, md_items, md_methods_table, md_prelude},
    ts::{ts_contract_methods, ts_extend_traits, ts_items, ts_prelude},
};
use std::{
    env,
    fs::File,
    io::{self, Read},
    path::Path,
};

/// Analyzes Rust source files to generate either TypeScript bindings or Markdown documentation
#[derive(Parser)]
#[clap(version = env!("CARGO_PKG_VERSION"), author = env!("CARGO_PKG_AUTHORS"))]
struct Args {
    #[clap(subcommand)]
    cmd: Cmd,
}

#[derive(Parser)]
enum Cmd {
    /// Emits TypeScript bindings
    #[clap(version = env!("CARGO_PKG_VERSION"), author = env!("CARGO_PKG_AUTHORS"))]
    TS(EmitArgs),

    /// Emits Markdown documentation
    #[clap(version = env!("CARGO_PKG_VERSION"), author = env!("CARGO_PKG_AUTHORS"))]
    MD(EmitArgs),
}

#[derive(Parser)]
struct EmitArgs {
    /// Does not emit date/time information,
    /// otherwise emits current time
    #[clap(long)]
    no_now: bool,

    /// Rust source files (*.rs) to analize
    #[clap()]
    files: Vec<String>,
}

fn main() {
    let args = Args::parse();

    match args.cmd {
        Cmd::TS(args) => emit_ts(args),
        Cmd::MD(args) => emit_md(args).unwrap(),
    }
}

fn emit_ts(args: EmitArgs) {
    let mut buf = std::io::stdout();
    ts_prelude(
        &mut buf,
        if args.no_now {
            "".to_string()
        } else {
            format!(" on {}", Utc::now())
        },
        env!("CARGO_BIN_NAME"),
    );

    let mut contract = Contract::new();

    for file_name in args.files {
        let ast = parse_rust(file_name);

        contract.forward_traits(&ast.items);
        ts_items(&mut buf, &ast.items, &contract);
    }

    ts_extend_traits(&mut buf, &contract);
    ts_contract_methods(&mut buf, &contract);
}

fn emit_md(args: EmitArgs) -> io::Result<()> {
    let mut buf = io::stdout();
    let utc_now = Utc::now();

    md_prelude(
        &mut buf,
        if args.no_now {
            "".to_string()
        } else {
            format!(" on {}", utc_now)
        },
    )?;

    let mut contract = Contract::new();

    let mut asts = Vec::new();
    for file_name in &args.files {
        let ast = parse_rust(file_name);
        contract.forward_traits(&ast.items);
        asts.push(ast);
    }
    md_methods_table(&mut buf, &asts, &contract);

    for file_name in &args.files {
        let ast = parse_rust(file_name);
        md_items(&ast, &contract);
    }

    md_footer(
        &mut buf,
        env!("CARGO_BIN_NAME"),
        if args.no_now {
            "".to_string()
        } else {
            format!(" *on {}*", utc_now)
        },
    )?;

    Ok(())
}

/// Returns the Rust syntax tree for the given `file_name` path.
/// Panics if the file cannot be open or the file has syntax errors.
///
/// ## Example
///
/// ```no_run
/// let mut ts = near_syn::ts::TS::new(std::io::stdout());
/// let ast = near_syn::parse_rust("path/to/file.rs");
/// ts.ts_items(&ast.items);
/// ```
fn parse_rust<S: AsRef<Path>>(file_name: S) -> syn::File {
    let mut file = File::open(file_name).expect("Unable to open file");
    let mut src = String::new();
    file.read_to_string(&mut src).expect("Unable to read file");

    syn::parse_file(&src).expect("Unable to parse file")
}
