use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[allow(deprecated)]
fn rloc() -> Command {
    Command::cargo_bin("rloc").unwrap()
}

fn create_test_project(dir: &std::path::Path) {
    fs::write(dir.join("main.rs"), "fn main() {\n    println!(\"Hello\");\n}\n").unwrap();
    fs::write(dir.join("lib.ts"), "// TypeScript\nconst x: number = 1;\n").unwrap();
    fs::write(dir.join("app.tsx"), "// TSX\nconst C = () => <div/>;\n").unwrap();
    fs::write(dir.join("script.py"), "# Python\nx = 1\n").unwrap();
}

#[test]
fn test_basic_run() {
    let temp = TempDir::new().unwrap();
    create_test_project(temp.path());

    rloc()
        .arg(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("files processed"));
}

#[test]
fn test_json_output() {
    let temp = TempDir::new().unwrap();
    create_test_project(temp.path());

    rloc()
        .arg(temp.path())
        .arg("--json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"header\""))
        .stdout(predicate::str::contains("\"nFiles\""));
}

#[test]
fn test_csv_output() {
    let temp = TempDir::new().unwrap();
    create_test_project(temp.path());

    rloc()
        .arg(temp.path())
        .arg("--csv")
        .assert()
        .success()
        .stdout(predicate::str::contains("Language,Files,Blank,Comment,Code"));
}

#[test]
fn test_yaml_output() {
    let temp = TempDir::new().unwrap();
    create_test_project(temp.path());

    rloc()
        .arg(temp.path())
        .arg("--yaml")
        .assert()
        .success()
        .stdout(predicate::str::contains("header:"))
        .stdout(predicate::str::contains("nFiles:"));
}

#[test]
fn test_markdown_output() {
    let temp = TempDir::new().unwrap();
    create_test_project(temp.path());

    rloc()
        .arg(temp.path())
        .arg("--md")
        .assert()
        .success()
        .stdout(predicate::str::contains("| Language |"));
}

#[test]
fn test_force_lang_lowercase() {
    let temp = TempDir::new().unwrap();
    fs::write(temp.path().join("app.tsx"), "const C = () => <div/>;").unwrap();

    // Using lowercase "typescript" should work (tests the bug fix)
    rloc()
        .arg(temp.path())
        .arg("--force-lang=typescript,tsx")
        .arg("--json")
        .assert()
        .success()
        .stdout(predicate::str::contains("TypeScript"));
}

#[test]
fn test_force_lang_exact_case() {
    let temp = TempDir::new().unwrap();
    fs::write(temp.path().join("app.tsx"), "const C = () => <div/>;").unwrap();

    rloc()
        .arg(temp.path())
        .arg("--force-lang=TypeScript,tsx")
        .arg("--json")
        .assert()
        .success()
        .stdout(predicate::str::contains("TypeScript"));
}

#[test]
fn test_include_lang() {
    let temp = TempDir::new().unwrap();
    create_test_project(temp.path());

    rloc()
        .arg(temp.path())
        .arg("--include-lang=Rust")
        .arg("--json")
        .assert()
        .success()
        .stdout(predicate::str::contains("Rust"))
        .stdout(predicate::str::contains("TypeScript").not());
}

#[test]
fn test_include_lang_case_insensitive() {
    let temp = TempDir::new().unwrap();
    create_test_project(temp.path());

    // Using lowercase should work
    rloc()
        .arg(temp.path())
        .arg("--include-lang=rust")
        .arg("--json")
        .assert()
        .success()
        .stdout(predicate::str::contains("Rust"));
}

#[test]
fn test_exclude_lang() {
    let temp = TempDir::new().unwrap();
    create_test_project(temp.path());

    rloc()
        .arg(temp.path())
        .arg("--exclude-lang=Python")
        .arg("--json")
        .assert()
        .success()
        .stdout(predicate::str::contains("Python").not());
}

#[test]
fn test_include_ext() {
    let temp = TempDir::new().unwrap();
    create_test_project(temp.path());

    rloc()
        .arg(temp.path())
        .arg("--include-ext=rs")
        .arg("--json")
        .assert()
        .success()
        .stdout(predicate::str::contains("Rust"))
        .stdout(predicate::str::contains("TypeScript").not());
}

#[test]
fn test_exclude_ext() {
    let temp = TempDir::new().unwrap();
    create_test_project(temp.path());

    rloc()
        .arg(temp.path())
        .arg("--exclude-ext=py")
        .arg("--json")
        .assert()
        .success()
        .stdout(predicate::str::contains("Python").not());
}

#[test]
fn test_by_file() {
    let temp = TempDir::new().unwrap();
    create_test_project(temp.path());

    rloc()
        .arg(temp.path())
        .arg("--by-file")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.rs"))
        .stdout(predicate::str::contains("lib.ts"));
}

#[test]
fn test_quiet_mode() {
    let temp = TempDir::new().unwrap();
    create_test_project(temp.path());

    rloc()
        .arg(temp.path())
        .arg("--quiet")
        .assert()
        .success();
}

#[test]
fn test_no_files_found() {
    let temp = TempDir::new().unwrap();
    // Empty directory

    rloc()
        .arg(temp.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("No source files found"));
}

#[test]
fn test_show_lang() {
    rloc()
        .arg("--show-lang")
        .assert()
        .success()
        .stdout(predicate::str::contains("Rust"))
        .stdout(predicate::str::contains("TypeScript"))
        .stdout(predicate::str::contains("Python"));
}

#[test]
fn test_show_ext() {
    rloc()
        .arg("--show-ext")
        .assert()
        .success()
        .stdout(predicate::str::contains("rs"))
        .stdout(predicate::str::contains("ts"))
        .stdout(predicate::str::contains("py"));
}

#[test]
fn test_sql_output() {
    let temp = TempDir::new().unwrap();
    create_test_project(temp.path());

    rloc()
        .arg(temp.path())
        .arg("--sql")
        .assert()
        .success()
        .stdout(predicate::str::contains("CREATE TABLE"))
        .stdout(predicate::str::contains("INSERT INTO"));
}

#[test]
fn test_xml_output() {
    let temp = TempDir::new().unwrap();
    create_test_project(temp.path());

    rloc()
        .arg(temp.path())
        .arg("--xml")
        .assert()
        .success()
        .stdout(predicate::str::contains("<?xml version"))
        .stdout(predicate::str::contains("<languages>"));
}
