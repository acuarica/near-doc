use assert_cmd::Command;

const NOW: &str = "123";

fn output(defs: &str, name: &str, view_methods: &str, change_methods: &str) -> String {
    format!(
        r#"// TypeScript bindings generated with near-ts v{} {} on {}

// Exports common NEAR Rust SDK types
export type U64 = string;
export type U128 = string;
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
        NOW,
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
    cmd.arg("--now").arg(NOW);
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
fn empty_files() {
    let mut cmd = near_ts();
    cmd.assert().code(0).stdout(output("", "", "", ""));
}

#[test]
fn basic_output() {
    use std::io::Write;
    use tempfile::NamedTempFile;

    let mut file = NamedTempFile::new().unwrap();
    writeln!(
        file,
        r#"
/// Doc-comment line 1 for A
/// Doc-comment line 2 for A
/// Doc-comment line 3 for A
#[derive(Serialize)]
struct A {{
    // No doc-comment for this field
    a1_field: U64,
    a2_field: U64,

    /// Line for a3
    /// Line for a2, then blank line
    ///
    /// Some markdown
    /// ```
    /// const a = [];
    /// const b = "";
    /// ```
    a3_field: U128,
}}

// No doc-comment for this struct
#[derive(Serialize)]
struct B {{
    b: U64,
}}

#[near_bindgen]
struct C {{
    f128: U128,
}}

#[near_bindgen]
impl C {{
    /// Line 1 for get_f128 first
    /// Line 2 for get_f128 second
    pub fn get_f128(&self) -> U128 {{
        self.f128
    }}

    // Regular comments are not transpiled
    pub fn set_f128(&mut self, value: U128) {{
        self.f128 = value;
    }}
}}

#[near_bindgen]
impl I for C {{
    /// Single-line comment for get
    fn get(&self) -> U128 {{
        self.f128
    }}
}}

// Omitted since near-bindgen is not present
impl J for C {{
    fn m() {{

    }}
}}
"#
    )
    .unwrap();
    file.flush().unwrap();
    let path = file.into_temp_path();

    let mut cmd = near_ts();
    cmd.arg(path.to_str().unwrap())
        .assert()
        .code(0)
        .stdout(output(r#"
/**
 *  Doc-comment line 1 for A
 *  Doc-comment line 2 for A
 *  Doc-comment line 3 for A
 */
export interface A {
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
export interface B {
    /**
     */
    b: U64;

}

export interface Self {
    /**
     *  Line 1 for get_f128 first
     *  Line 2 for get_f128 second
     */
    get_f128(): Promise<U128>;

    /**
     */
    set_f128(args: { value: U128 }): Promise<void>;

}

export interface I {
    /**
     *  Single-line comment for get
     */
    get(): Promise<U128>;

}

export type C = Self & I;
"#, "C", "get_f128,get", "set_f128"));

    path.close().unwrap();
}
