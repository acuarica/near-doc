# near-syn

[![Build Status](https://github.com/acuarica/near-syn/actions/workflows/near-syn.yml/badge.svg)](https://github.com/acuarica/near-syn/actions/)
[![Crates.io](https://img.shields.io/crates/v/near-syn)](https://crates.io/crates/near-syn/)
[![docs.rs](https://img.shields.io/docsrs/near-syn)](https://docs.rs/near-syn/)
![License](https://img.shields.io/crates/l/near-syn.svg)

`near-syn` is a library and command line utilities to
make contract development for the NEAR platform easier.

The `near-syn` package contains two command line utilities:

- `near-ts` generates TypeScript bindings from Rust source files.
- `near-doc` generates Markdown documentation from Rust source files.

## Installation

To install the `near-syn` command line utilities use

```sh
cargo install near-syn
```

Or alternatively you can install it directly from GitHub

```sh
cargo install --git https://github.com/acuarica/near-syn --branch main
```

## Usage

The `near-ts` utility takes a group of Rust source files,
and outputs the generated TypeScript bindings.

```sh
near-ts path/to/src/lib.rs > src/contract.ts
```

Similarly, the `near-doc` utility takes a group of Rust source files,
and outputs the generated Markdown documentation.

```sh
near-doc path/to/src/lib.rs > path/to/README.md
```
