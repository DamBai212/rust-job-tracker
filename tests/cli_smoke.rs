use assert_cmd::cargo::cargo_bin_cmd;
use predicates::str::contains;
use tempfile::tempdir;

#[test]
fn delete_job_isolated_db() {
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
    add.assert().success();

    let mut del = cargo_bin_cmd!("rust-job-tracker");
    del.args([
        "--db-path",
        db_path.to_str().unwrap(),
        "delete",
        "--id",
        "1",
    ]);
    del.assert().success().stdout(contains("Deleted job #1"));

    let mut list = cargo_bin_cmd!("rust-job-tracker");
    list.args(["--db-path", db_path.to_str().unwrap(), "list"]);
    list.assert().success().stdout(contains("No jobs"));
}
