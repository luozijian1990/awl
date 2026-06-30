use std::fs;

#[test]
fn cli_help_prints_usage_and_does_not_open_database() {
    let db_path = std::env::temp_dir().join(format!("worklog-cli-{}.db", uuid::Uuid::new_v4()));
    let mut stdout = Vec::new();

    let code = super::run_with_args(
        vec!["worklog", "--db", db_path.to_str().unwrap(), "--help"],
        &mut stdout,
    )
    .unwrap();

    let output = String::from_utf8(stdout).unwrap();
    assert_eq!(code, 0);
    assert!(output.contains("Usage: worklog [--db <path>] <command>"));
    assert!(output.contains("--help"));
    assert!(output.contains("entry add"));
    assert!(output.contains("calendar list"));
    assert!(output.contains("export report-source"));
    assert!(
        !db_path.exists(),
        "help should not open or create the database"
    );
}

#[test]
fn cli_add_list_confirm_and_export_report_source() {
    let db_path = std::env::temp_dir().join(format!("worklog-cli-{}.db", uuid::Uuid::new_v4()));
    let mut stdout = Vec::new();

    let id = super::run_with_args(
        vec![
            "worklog",
            "--db",
            db_path.to_str().unwrap(),
            "entry",
            "add",
            "--title",
            "AI draft",
            "--start",
            "2026-06-16T09:00:00Z",
            "--end",
            "2026-06-16T10:00:00Z",
            "--actor",
            "ai",
            "--source",
            "codex",
        ],
        &mut stdout,
    )
    .unwrap();
    assert_eq!(id, 0);
    let output = String::from_utf8(stdout.clone()).unwrap();
    assert!(output.contains("AI draft"));
    assert!(output.contains("draft"));

    stdout.clear();
    super::run_with_args(
        vec![
            "worklog",
            "--db",
            db_path.to_str().unwrap(),
            "entry",
            "list",
            "--status",
            "draft",
            "--format",
            "json",
        ],
        &mut stdout,
    )
    .unwrap();
    let list_json: serde_json::Value = serde_json::from_slice(&stdout).unwrap();
    let entry_id = list_json["entries"][0]["id"].as_i64().unwrap();
    assert_eq!(list_json["entries"][0]["status"], "draft");

    stdout.clear();
    super::run_with_args(
        vec![
            "worklog",
            "--db",
            db_path.to_str().unwrap(),
            "entry",
            "confirm",
            &entry_id.to_string(),
        ],
        &mut stdout,
    )
    .unwrap();

    stdout.clear();
    super::run_with_args(
        vec![
            "worklog",
            "--db",
            db_path.to_str().unwrap(),
            "export",
            "report-source",
            "--start",
            "2026-06-16T00:00:00Z",
            "--end",
            "2026-06-17T00:00:00Z",
            "--format",
            "markdown",
        ],
        &mut stdout,
    )
    .unwrap();
    assert_eq!(
        String::from_utf8(stdout.clone()).unwrap(),
        "1. AI draft\n\n"
    );

    stdout.clear();
    super::run_with_args(
        vec![
            "worklog",
            "--db",
            db_path.to_str().unwrap(),
            "export",
            "report-source",
            "--start",
            "2026-06-16T00:00:00Z",
            "--end",
            "2026-06-17T00:00:00Z",
            "--format",
            "json",
        ],
        &mut stdout,
    )
    .unwrap();
    let export_json: serde_json::Value = serde_json::from_slice(&stdout).unwrap();
    assert_eq!(export_json["items"][0]["title"], "AI draft");

    let _ = fs::remove_file(db_path);
}

#[test]
fn cli_calendar_commands_and_export_output_file_work() {
    let db_path = std::env::temp_dir().join(format!("worklog-cli-{}.db", uuid::Uuid::new_v4()));
    let output_path =
        std::env::temp_dir().join(format!("worklog-report-{}.md", uuid::Uuid::new_v4()));
    let mut stdout = Vec::new();

    super::run_with_args(
        vec![
            "worklog",
            "--db",
            db_path.to_str().unwrap(),
            "calendar",
            "add",
            "--name",
            "Reports",
            "--color",
            "#00aa00",
            "--default",
        ],
        &mut stdout,
    )
    .unwrap();
    let calendar: serde_json::Value = serde_json::from_slice(&stdout).unwrap();
    assert_eq!(calendar["name"], "Reports");
    assert_eq!(calendar["is_default"], true);
    let calendar_id = calendar["id"].as_i64().unwrap();

    stdout.clear();
    super::run_with_args(
        vec![
            "worklog",
            "--db",
            db_path.to_str().unwrap(),
            "calendar",
            "list",
        ],
        &mut stdout,
    )
    .unwrap();
    let calendars: serde_json::Value = serde_json::from_slice(&stdout).unwrap();
    assert_eq!(calendars.as_array().unwrap().len(), 1);

    stdout.clear();
    super::run_with_args(
        vec![
            "worklog",
            "--db",
            db_path.to_str().unwrap(),
            "calendar",
            "default",
            &calendar_id.to_string(),
        ],
        &mut stdout,
    )
    .unwrap();

    stdout.clear();
    super::run_with_args(
        vec![
            "worklog",
            "--db",
            db_path.to_str().unwrap(),
            "entry",
            "add",
            "--title",
            "Confirmed item",
            "--start",
            "2026-06-16T09:00:00Z",
            "--end",
            "2026-06-16T10:00:00Z",
            "--actor",
            "human",
            "--source",
            "manual",
            "--status",
            "confirmed",
            "--calendar",
            &calendar_id.to_string(),
        ],
        &mut stdout,
    )
    .unwrap();

    stdout.clear();
    super::run_with_args(
        vec![
            "worklog",
            "--db",
            db_path.to_str().unwrap(),
            "export",
            "report-source",
            "--start",
            "2026-06-16T00:00:00Z",
            "--end",
            "2026-06-17T00:00:00Z",
            "--format",
            "markdown",
            "--output",
            output_path.to_str().unwrap(),
        ],
        &mut stdout,
    )
    .unwrap();
    assert!(stdout.is_empty());
    assert_eq!(
        fs::read_to_string(&output_path).unwrap(),
        "1. Confirmed item\n"
    );

    let _ = fs::remove_file(db_path);
    let _ = fs::remove_file(output_path);
}
