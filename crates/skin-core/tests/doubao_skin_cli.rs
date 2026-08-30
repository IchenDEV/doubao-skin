use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

fn binary() -> &'static str {
    env!("CARGO_BIN_EXE_doubao-skin")
}

fn temporary_test_dir(label: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "doubao-skin-cli-{label}-{}-{stamp}",
        std::process::id()
    ));
    fs::create_dir_all(&path).unwrap();
    path
}

fn run(root: &Path, args: &[&str]) -> Output {
    Command::new(binary())
        .args(args)
        .env("DOUBAO_SKIN_THEMES_DIR", root.join("bundled"))
        .env("DOUBAO_SKIN_USER_THEMES_DIR", root.join("installed"))
        .env("DOUBAO_SKIN_DOUBAO_CDP_PORT", "9")
        .env("DOUBAO_SKIN_DOUBAO_WORK_CDP_PORT", "9")
        .output()
        .unwrap()
}

#[test]
fn help_and_argument_errors_use_the_stable_exit_contract() {
    let root = temporary_test_dir("args");
    let help = run(&root, &["--help"]);
    assert!(help.status.success());
    let help_text = String::from_utf8_lossy(&help.stdout);
    assert!(help_text.contains("doubao-skin"));
    for command in [
        "list",
        "create",
        "check",
        "preview",
        "pack",
        "install",
        "apply",
        "restore",
        "build",
        "remove-build",
    ] {
        assert!(help_text.contains(command), "help is missing {command}");
    }

    let unknown = run(&root, &["unknown", "--json"]);
    assert_eq!(unknown.status.code(), Some(2));
    let envelope: serde_json::Value = serde_json::from_slice(&unknown.stdout).unwrap();
    assert_eq!(envelope["ok"], false);
    assert_eq!(envelope["command"], "unknown");
    assert_eq!(envelope["error"]["code"], "arguments");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn side_effecting_commands_have_isolated_json_and_exit_contracts() {
    let root = temporary_test_dir("side-effects");
    let cases = [
        (
            vec![
                "apply",
                "missing-theme",
                "--target",
                "doubao-work",
                "--json",
            ],
            3,
            "apply",
            "invalid-theme",
        ),
        (
            vec!["restore", "--target", "doubao-work", "--json"],
            4,
            "restore",
            "external-operation",
        ),
        (
            vec!["build", "missing-theme", "--json"],
            3,
            "build",
            "invalid-theme",
        ),
        (
            vec!["remove-build", "unexpected", "--json"],
            2,
            "remove-build",
            "arguments",
        ),
    ];

    for (args, exit_code, command, error_code) in cases {
        let output = run(&root, &args);
        assert_eq!(
            output.status.code(),
            Some(exit_code),
            "{command}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let envelope: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(envelope["ok"], false);
        assert_eq!(envelope["command"], command);
        assert_eq!(envelope["error"]["code"], error_code);
    }

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn cli_completes_create_check_preview_pack_install_and_list() {
    let root = temporary_test_dir("roundtrip");
    let theme_dir = root.join("evening-amber");
    let theme = theme_dir.to_string_lossy().into_owned();
    let package = root.join("evening-amber.doubao-skin.zip");
    let package_text = package.to_string_lossy().into_owned();

    let create = run(
        &root,
        &[
            "create",
            &theme,
            "--name",
            "晚风琥珀",
            "--description",
            "温暖安静，适合夜间阅读",
            "--accent",
            "#d58a32",
            "--appearance",
            "dark",
            "--author",
            "测试作者",
            "--json",
        ],
    );
    assert!(
        create.status.success(),
        "{}",
        String::from_utf8_lossy(&create.stderr)
    );
    let created: serde_json::Value = serde_json::from_slice(&create.stdout).unwrap();
    assert_eq!(created["ok"], true);
    assert_eq!(created["command"], "create");
    assert_eq!(created["result"]["id"], "evening-amber");

    for args in [
        vec!["check", &theme, "--json"],
        vec!["preview", &theme, "--json"],
        vec!["pack", &theme, &package_text, "--json"],
    ] {
        let output = run(&root, &args);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let envelope: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(envelope["ok"], true);
    }

    let install = run(&root, &["install", &package_text, "--json"]);
    assert!(
        install.status.success(),
        "{}",
        String::from_utf8_lossy(&install.stderr)
    );
    let installed: serde_json::Value = serde_json::from_slice(&install.stdout).unwrap();
    assert_eq!(installed["result"]["id"], "evening-amber");

    let list = run(&root, &["list", "--json"]);
    assert!(list.status.success());
    let listed: serde_json::Value = serde_json::from_slice(&list.stdout).unwrap();
    assert!(listed["result"]["themes"]
        .as_array()
        .unwrap()
        .iter()
        .any(|theme| theme["id"] == "evening-amber"));

    let text_list = run(&root, &["list"]);
    assert!(text_list.status.success());
    assert!(String::from_utf8_lossy(&text_list.stdout).contains("- 晚风琥珀 (evening-amber)"));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn invalid_theme_and_external_failure_have_distinct_exit_codes() {
    let root = temporary_test_dir("errors");
    let missing = root.join("missing-theme").to_string_lossy().into_owned();
    let invalid = run(&root, &["check", &missing, "--json"]);
    assert_eq!(invalid.status.code(), Some(3));
    let invalid_json: serde_json::Value = serde_json::from_slice(&invalid.stdout).unwrap();
    assert_eq!(invalid_json["error"]["code"], "invalid-theme");

    let restore = run(&root, &["restore", "--target", "doubao-work", "--json"]);
    assert_eq!(restore.status.code(), Some(4));
    let restore_json: serde_json::Value = serde_json::from_slice(&restore.stdout).unwrap();
    assert_eq!(restore_json["error"]["code"], "external-operation");

    let blocking_file = root.join("blocking-file");
    fs::write(&blocking_file, "not a directory").unwrap();
    let blocked_theme = blocking_file.join("new-theme");
    let blocked_theme = blocked_theme.to_string_lossy().into_owned();
    let filesystem = run(
        &root,
        &["create", &blocked_theme, "--name", "文件系统失败", "--json"],
    );
    assert_eq!(filesystem.status.code(), Some(4));
    let filesystem_json: serde_json::Value = serde_json::from_slice(&filesystem.stdout).unwrap();
    assert_eq!(filesystem_json["error"]["code"], "external-operation");
    fs::remove_dir_all(root).unwrap();
}
