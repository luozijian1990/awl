use std::fs;
use std::path::PathBuf;

#[test]
fn planning_docs_are_not_ignored_and_workspace_contains_core_and_cli() {
    let root = workspace_root();

    let gitignore = fs::read_to_string(root.join(".gitignore")).unwrap();
    let ignore_rules = gitignore
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect::<Vec<_>>();
    assert!(
        !ignore_rules.iter().any(|line| line.contains("docs/")),
        "planning docs must remain tracked"
    );

    let cargo_toml = fs::read_to_string(root.join("Cargo.toml")).unwrap();
    assert!(cargo_toml.contains("crates/worklog-core"));
    assert!(cargo_toml.contains("crates/worklog-cli"));
}

#[test]
fn make_entrypoints_exist_for_common_development_tasks() {
    let root = workspace_root();

    let makefile = fs::read_to_string(root.join("Makefile")).unwrap();
    for target in ["fmt:", "test:", "build:", "smoke:"] {
        assert!(makefile.contains(target), "Makefile missing {target}");
    }

    let make_bat = fs::read_to_string(root.join("make.bat")).unwrap();
    for target in ["fmt", "test", "build", "smoke"] {
        assert!(
            make_bat.contains(target),
            "make.bat missing {target} command"
        );
    }
}

#[test]
fn desktop_tauri_shell_is_scaffolded_and_joined_to_workspace() {
    let root = workspace_root();

    let cargo_toml = fs::read_to_string(root.join("Cargo.toml")).unwrap();
    assert!(cargo_toml.contains("\"desktop/src-tauri\""));

    for path in [
        "desktop/package.json",
        "desktop/index.html",
        "desktop/tsconfig.json",
        "desktop/vite.config.ts",
        "desktop/src/main.tsx",
        "desktop/src-tauri/Cargo.toml",
        "desktop/src-tauri/tauri.conf.json",
        "desktop/src-tauri/src/lib.rs",
        "desktop/src-tauri/src/main.rs",
    ] {
        assert!(
            root.join(path).exists(),
            "missing desktop scaffold file: {path}"
        );
    }

    let package_json = fs::read_to_string(root.join("desktop/package.json")).unwrap();
    assert!(package_json.contains("@tauri-apps/api"));
    assert!(package_json.contains("@tauri-apps/cli"));

    let tauri_cargo = fs::read_to_string(root.join("desktop/src-tauri/Cargo.toml")).unwrap();
    assert!(tauri_cargo.contains("worklog-core"));
}

#[test]
fn release_workflow_uploads_windows_portable_desktop_zip() {
    let root = workspace_root();

    let workflow = fs::read_to_string(root.join(".github/workflows/release.yml")).unwrap();
    assert!(workflow.contains("Package portable ZIP"));
    assert!(workflow.contains("target/release/worklog-desktop.exe"));
    assert!(workflow.contains("Compress-Archive"));
    assert!(workflow.contains("dist/Worklog_${version}_x64-portable.zip"));
    assert!(workflow.contains("dist/*-portable.zip"));
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}
