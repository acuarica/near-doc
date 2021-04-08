# near-doc

Command utility to generate Markdown documentation from a Rust contract for the NEAR blockchain.

## Installation

To install the `near-doc` utility

```sh
cargo install --git https://github.com/acuarica/near-doc --branch main
```

## Usage

The `near-doc` utility takes a Rust source file,
and outputs the generated Markdown documentation.

```sh
near-doc path/to/src/lib.rs > path/to/README.md
```
