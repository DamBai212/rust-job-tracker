use assert_cmd::cargo::cargo_bin_cmd;
use predicates::str::contains;
use tempfile::tempdir;

#[test]
fn add_then_list_isolated_db() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");

    let mut add = cargo_bin_cmd!("rust-job-tracker");
    add.args([
        "--db-path",
        db_path.to_str().unwrap(),
        "add",
        "--company",
        "Acme",
        "--role",
        "Backend Engineer",
        "--status",
        "applied",
    ]);
    add.assert().success().stdout(contains("Added job"));

    let mut upd = cargo_bin_cmd!("rust-job-tracker");
    upd.args([
        "--db-path",
        db_path.to_str().unwrap(),
        "update-status",
        "--id",
        "1",
        "--status",
        "interviewing",
    ]);
    upd.assert().success().stdout(contains("Updated job #1"));

    let mut list = cargo_bin_cmd!("rust-job-tracker");
    list.args(["--db-path", db_path.to_str().unwrap(), "list"]);
    list.assert().success().stdout(contains("interviewing"));
}
