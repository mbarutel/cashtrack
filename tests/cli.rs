use assert_cmd::Command;
use tempfile::TempDir;

struct TestEnv {
    _dir: TempDir,
    config: std::path::PathBuf,
    db: std::path::PathBuf,
}

fn setup() -> TestEnv {
    let dir = TempDir::new().unwrap();
    let config = dir.path().join("categories.yaml");
    let db = dir.path().join("test.db");
    std::fs::write(&config, include_str!("fixtures/categories.yaml")).unwrap();

    TestEnv {
        _dir: dir,
        config,
        db,
    }
}

fn cashtrack(env: &TestEnv) -> Command {
    let mut cmd = Command::cargo_bin("cashtrack").unwrap();
    cmd.env("CASHTRACK_CONFIG", &env.config)
        .env("CASHTRACK_DB", &env.db);
    cmd
}

#[test]
fn import_then_list() {
    let env = setup();

    cashtrack(&env)
        .args([
            "import",
            "--csv-path",
            "tests/fixtures/test_transactions_export.csv",
        ])
        .assert()
        .success();

    cashtrack(&env)
        .args(["list"])
        .assert()
        .success()
        .stdout(predicates::str::contains("Outflow"));
}
