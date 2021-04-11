use std::io::Write;
use tempfile::{NamedTempFile, TempPath};

pub fn rust_test_file() -> TempPath {
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

    file.into_temp_path()
}
