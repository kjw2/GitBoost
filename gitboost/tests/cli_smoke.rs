use assert_cmd::Command;
use predicates::str::contains;
use std::fs;
use tempfile::TempDir;

fn gitboost() -> Command {
    Command::cargo_bin("gitboost").expect("gitboost 바이너리를 찾을 수 없습니다")
}

/// `gitboost --version` → 종료 코드 0, "gitboost" 포함
#[test]
fn version_flag_prints_version() {
    gitboost()
        .arg("--version")
        .assert()
        .success()
        .stdout(contains("gitboost"));
}

/// `gitboost --help` → 종료 코드 0
#[test]
fn help_flag_prints_help() {
    gitboost().arg("--help").assert().success();
}

/// `gitboost create` (이름 없이) → 종료 코드 2
#[test]
fn create_without_name_fails() {
    gitboost().arg("create").assert().code(2);
}

/// `gitboost create "in valid"` → 0이 아닌 종료 코드 + 에러 메시지에 "이름" 포함
#[test]
fn create_invalid_name_fails() {
    gitboost()
        .args(["create", "in valid"])
        .assert()
        .failure()
        .stderr(contains("이름"));
}

/// 비어있지 않은 디렉토리가 있을 때 → 종료 코드 13 (인증 단계 진입 전)
#[test]
fn create_existing_nonempty_dir_fails() {
    let tmp = TempDir::new().unwrap();
    let project_dir = tmp.path().join("my-project");
    fs::create_dir_all(&project_dir).unwrap();
    // 파일을 하나 넣어 non-empty로 만들기
    fs::write(project_dir.join("existing.txt"), "hello").unwrap();

    gitboost()
        .args(["create", "my-project"])
        .current_dir(tmp.path())
        .assert()
        .code(13);
}

/// `gitboost --help` 출력에 4개 서브커맨드가 모두 포함
#[test]
fn help_shows_all_subcommands() {
    gitboost()
        .arg("--help")
        .assert()
        .success()
        .stdout(contains("create"))
        .stdout(contains("login"))
        .stdout(contains("logout"))
        .stdout(contains("whoami"));
}
