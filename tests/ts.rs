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

    // let mut file = tempfile().unwrap();
    let mut file = NamedTempFile::new().unwrap();
    writeln!(
        file,
        r#"
#[derive(Serialize)]
struct A {{
    field: U64,
}}

#[near_bindgen]
struct C {{
    f128: U128,
}}

#[near_bindgen]
impl C {{
    pub fn get_f128(&self) -> U128 {{
        self.f128
    }}
    pub fn set_f128(&mut self, value: U128) {{
        self.f128 = value;
    }}
}}

#[near_bindgen]
impl I for C {{
    pub fn get(&self) -> U128 {{
        self.f128
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
 */
export interface A {
    /**
     */
    field: U64;
}

export interface Self {
    /**
     */
    get_f128(): Promise<U128>;

    /**
     */
    set_f128(args: { value: U128 }): Promise<void>;

}

export interface I {
    /**
     */
    get(): Promise<U128>;

}

export type C = Self & I;
"#, "C", "get_f128,get", "set_f128"));

    path.close().unwrap();
}
