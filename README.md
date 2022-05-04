# near-syn

[![Build Status](https://github.com/acuarica/near-syn/actions/workflows/near-syn.yml/badge.svg)](https://github.com/acuarica/near-syn/actions/)
[![Crates.io](https://img.shields.io/crates/v/near-syn)](https://crates.io/crates/near-syn/)
[![docs.rs](https://img.shields.io/docsrs/near-syn)](https://docs.rs/near-syn/)
![License](https://img.shields.io/crates/l/near-syn.svg)

`near-syn` is a library and command line utility to ease contract development for the [NEAR Protocol](https://near.org/).
It leverages Rust `syn` to generate TypeScript bindings and Markdown docs.

The `near-syn` command line utility contains two sub-commands:

- `ts` generates TypeScript bindings from Rust source files.
- `md` generates Markdown documentation from Rust source files.

For more details see `near-syn --help`.

## Installation

To install the `near-syn` command line utilities use

```sh
cargo install near-syn
```

Or alternatively you can `install` it directly from GitHub (see more [`install` options](https://doc.rust-lang.org/cargo/commands/cargo-install.html#install-options))

```sh
cargo install --git https://github.com/acuarica/near-syn --branch main
```

## Usage

The `near-syn ts` utility takes a group of Rust source files,
and outputs the generated TypeScript bindings.

```sh
near-syn ts path/to/src/lib.rs > src/contract.ts
```

Similarly, the `near-syn md` utility takes a group of Rust source files,
and outputs the generated Markdown documentation.

```sh
near-syn md path/to/src/lib.rs > path/to/README.md
```

## Publishing

We use [`cargo-release`](https://github.com/crate-ci/cargo-release) to verify, publish and tag new versions.
First, install

```sh
cargo install cargo-release
```

Make sure you have logged in with `cargo`

```sh
cargo login
```

To perform a dry-run

```sh
cargo release --verbose [LEVEL]
```

And to actually publish, tag and release a new version, run

```sh
cargo release --verbose --execute [LEVEL]
```

where `[LEVEl]` is the [bump level](https://github.com/crate-ci/cargo-release/blob/master/docs/reference.md#bump-level) incremented to get a new version.
For example

```sh
cargo release --verbose --execute patch
```
