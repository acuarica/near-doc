use assert_cmd::Command;
use input::rust_test_files;

mod input;

fn output(defs: &str, name: &str, view_methods: &str, change_methods: &str) -> String {
    format!(
        r#"// TypeScript bindings generated with near-ts v{} {}

// Exports common NEAR Rust SDK types
export type U64 = string;
export type I64 = string;
export type U128 = string;
export type I128 = string;
export type AccountId = string;
export type ValidAccountId = string;
{}
export const {}Methods = {{
    viewMethods: [{}
    ],
    changeMethods: [{}
    ],
}};
"#,
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_REPOSITORY"),
        defs,
        name,
        view_methods
            .split_terminator(",")
            .map(|s| format!("\n        {:?},", s))
            .collect::<Vec<String>>()
            .join(""),
        change_methods
            .split_terminator(",")
            .map(|s| format!("\n        {:?},", s))
            .collect::<Vec<String>>()
            .join(""),
    )
}

fn near_ts() -> Command {
    let mut cmd = Command::cargo_bin("near-ts").unwrap();
    cmd.arg("--no-now");
    cmd
}

#[test]
fn check_version() {
    let mut cmd = near_ts();
    cmd.arg("--version")
        .assert()
        .code(0)
        .stdout(format!("near-ts {}\n", env!("CARGO_PKG_VERSION")));
}

#[test]
fn transpile_zero_rust_files_to_ts() {
    let mut cmd = near_ts();
    cmd.assert().code(0).stdout(output("", "", "", ""));
}

#[test]
fn transpile_single_rust_file_to_ts() {
    let paths = rust_test_files();

    let mut cmd = near_ts();
    cmd.arg(paths[0].to_str().unwrap())
        .assert()
        .code(0)
        .stdout(output(
            r#"
/**
 */
export type AType = number;

/**
 *  Doc-comments for a type def
 */
export type BType = number;

/**
 *  Doc-comment line 1 for A
 *  Doc-comment line 2 for A
 *  Doc-comment line 3 for A
 */
export type A = {
    /**
     */
    a1_field: U64;

    /**
     */
    a2_field: U64;

    /**
     *  Line for a3
     *  Line for a2, then blank line
     * 
     *  Some markdown
     *  ```
     *  const a = [];
     *  const b = "";
     *  ```
     */
    a3_field: U128;

}

/**
 */
export type B = {
    /**
     */
    b: U64;

}

/**
 *  doc-comment for enum
 */
export enum E {
    /**
     */
    V1,

    /**
     */
    V2,

}

/**
 */
export interface C {
    /**
     *  init func
     */
    init_here: { f128: U128 };

    /**
     *  Line 1 for get_f128 first
     *  Line 2 for get_f128 second
     */
    get_f128(): Promise<U128>;

    /**
     *  Set f128.
     */
    set_f128(args: { value: U128 }, gas?: any): Promise<void>;

    /**
     */
    get_f128_other_way(args: { key: U128 }): Promise<U128>;

    /**
     */
    more_types(args: { key: U128, tuple: [string, number[]] }, gas?: any): Promise<void>;

    /**
     *  Pay to set f128.
     */
    set_f128_with_sum(args: { a_value: U128, other_value: U128 }, gas?: any, amount?: any): Promise<void>;

}

/**
 */
export interface C {
    /**
     *  another impl
     */
    another_impl(args: { f128: U128 }): Promise<U128>;

}

/**
 */
export interface I {
    /**
     *  Single-line comment for get
     */
    get(): Promise<U128>;

}

/**
 */
export type A_in_mod = number;

export interface C extends I {}
"#,
            "C",
            "get_f128,get_f128_other_way,another_impl,get",
            "set_f128,more_types,set_f128_with_sum",
        ));

    paths.into_iter().for_each(|path| path.close().unwrap());
}

#[test]
fn transpile_multiple_rust_files_to_ts() {
    let paths = rust_test_files();

    let mut cmd = near_ts();
    cmd.args(&paths[1..]).assert().code(0).stdout(output(
        r#"
/**
 */
export type S = {
    /**
     */
    f: number;

}

/**
 */
export interface S {
    /**
     */
    get(): Promise<number>;

}

/**
 */
export type T = [number, boolean];

/**
 */
export type U = AccountId;
"#,
        "S",
        "get",
        "",
    ));

    paths.into_iter().for_each(|path| path.close().unwrap());
}
