use assert_cmd::Command;
use std::io::Write;
use tempfile::{NamedTempFile, TempPath};

fn rust_test_files() -> Vec<TempPath> {
    vec![
        include_str!("input/input1.rs"),
        include_str!("input/input2.rs"),
        include_str!("input/input3.rs"),
    ]
    .into_iter()
    .map(|content| {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "{}", content).unwrap();
        file.flush().unwrap();
        file.into_temp_path()
    })
    .collect()
}

fn near_cmd(command: &str) -> Command {
    let mut cmd = Command::cargo_bin("near-syn").unwrap();
    cmd.arg(command);
    cmd.arg("--no-now");
    cmd
}

#[test]
fn check_version() {
    let mut cmd = Command::cargo_bin("near-syn").unwrap();
    cmd.arg("--version")
        .assert()
        .code(0)
        .stdout(format!("near-syn {}\n", env!("CARGO_PKG_VERSION")));
}

mod ts {

    use super::{near_cmd, rust_test_files};
    use assert_cmd::Command;

    fn output(defs: &str, name: &str, view_methods: &str, change_methods: &str) -> String {
        format!(
            include_str!("input/_template.ts"),
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
        near_cmd("ts")
    }

    #[test]
    fn transpile_zero_rust_files_to_ts() {
        near_ts().assert().code(0).stdout(output("", "", "", ""));
    }

    #[test]
    fn transpile_single_rust_file_to_ts() {
        let paths = rust_test_files();

        near_ts()
            .arg(paths[0].to_str().unwrap())
            .assert()
            .code(0)
            .stdout(output(
                include_str!("input/output1.ts"),
                "C",
                "get_f128,get_f128_other_way,another_impl,get",
                "set_f128,more_types,set_f128_with_sum",
            ));

        paths.into_iter().for_each(|path| path.close().unwrap());
    }

    #[test]
    fn transpile_multiple_rust_files_to_ts() {
        let paths = rust_test_files();

        near_ts().args(&paths[1..]).assert().code(0).stdout(output(
            include_str!("input/output2.ts"),
            "S",
            "get",
            "",
        ));

        paths.into_iter().for_each(|path| path.close().unwrap());
    }
}

mod md {

    use super::{near_cmd, rust_test_files};
    use assert_cmd::Command;

    fn output(init_methods: &str, view_methods: &str, change_methods: &str, text: &str) -> String {
        format!(
            include_str!("input/_template.md"),
            init_methods,
            view_methods,
            change_methods,
            text,
            env!("CARGO_PKG_VERSION"),
            env!("CARGO_PKG_REPOSITORY"),
        )
    }

    fn near_md() -> Command {
        near_cmd("md")
    }

    #[test]
    fn transpile_zero_rust_files_to_md() {
        near_md().assert().code(0).stdout(output("", "", "", ""));
    }

    #[test]
    fn transpile_single_rust_file_to_md() {
        let paths = rust_test_files();

        near_md()
            .arg(paths[0].to_str().unwrap())
            .assert()
            .code(0)
            .stdout(output(
                r#"| :rocket: `init_here` (_constructor_) |  init func | `Self` |
"#, 
                r#"| :eyeglasses: `get_f128` |  Line 1 for get_f128 first  Line 2 for get_f128 second | `U128` |
| :eyeglasses: `get_f128_other_way` |  | `U128` |
| :eyeglasses: `another_impl` |  another impl | `U128` |
| :eyeglasses: `get` |  Single-line comment for get | `U128` |
"#, 
                r#"| :writing_hand: `set_f128` |  Set f128. | `void` |
| :writing_hand: `more_types` |  | `void` |
| &#x24C3; `set_f128_with_sum` |  Pay to set f128. | `void` |
"#, 
                include_str!("input/output2.md")));

        paths.into_iter().for_each(|path| path.close().unwrap());
    }
}
