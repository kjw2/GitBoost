pub mod credential;
pub mod runner;

use crate::error::{GitBoostError, Result};
use runner::GitRunner;
use std::path::Path;

/// 디렉토리에 git 저장소를 초기화합니다.
///
/// `--initial-branch=main`을 지원하지 않는 구버전 git의 경우
/// 초기화 후 symbolic-ref로 폴백합니다.
pub fn init_repo(cwd: &Path) -> Result<()> {
    let runner = GitRunner::new(cwd);

    // git init --initial-branch=main 시도
    match runner.run(&["init", "--initial-branch=main"]) {
        Ok(_) => return Ok(()),
        Err(_) => {
            tracing::debug!("--initial-branch 미지원, git init 후 symbolic-ref 폴백 사용");
        }
    }

    // 폴백: git init 후 기본 브랜치를 main으로 설정
    runner.run(&["init"])?;
    runner.run(&["symbolic-ref", "HEAD", "refs/heads/main"])?;
    Ok(())
}

/// 모든 파일을 스테이징합니다.
pub fn stage_all(cwd: &Path) -> Result<()> {
    GitRunner::new(cwd).run(&["add", "."])?;
    Ok(())
}

/// GPG 서명 없이 초기 커밋을 생성합니다.
///
/// user.name과 user.email이 지정된 경우 해당 값을 사용합니다.
pub fn initial_commit(cwd: &Path, message: &str, name: &str, email: &str) -> Result<()> {
    let runner = GitRunner::new(cwd);
    runner.run(&[
        "-c",
        "commit.gpgsign=false",
        "-c",
        &format!("user.name={}", name),
        "-c",
        &format!("user.email={}", email),
        "commit",
        "-m",
        message,
    ])?;
    Ok(())
}

/// 원격 저장소를 등록합니다.
pub fn add_remote(cwd: &Path, name: &str, url: &str) -> Result<()> {
    GitRunner::new(cwd).run(&["remote", "add", name, url])?;
    Ok(())
}

/// 토큰 인증으로 원격 저장소에 push합니다.
pub fn push(cwd: &Path, token: &str, remote: &str, branch: &str) -> Result<()> {
    credential::push_with_token(cwd, token, remote, branch)
}

/// 로컬 git config에서 user.name을 읽습니다.
pub fn get_git_user_name(cwd: &Path) -> Option<String> {
    GitRunner::new(cwd)
        .run(&["config", "user.name"])
        .ok()
        .filter(|s| !s.is_empty())
}

/// 로컬 git config에서 user.email을 읽습니다.
pub fn get_git_user_email(cwd: &Path) -> Option<String> {
    GitRunner::new(cwd)
        .run(&["config", "user.email"])
        .ok()
        .filter(|s| !s.is_empty())
}

/// 저장소 이름이 유효한지 검증합니다.
pub fn validate_repo_name(name: &str) -> Result<()> {
    if !crate::cli::validate_repo_name(name) {
        return Err(GitBoostError::Prerequisite(format!(
            "유효하지 않은 저장소 이름: '{}'\n  \
             영문, 숫자, 하이픈(-), 언더스코어(_), 점(.)만 허용되며 최대 100자입니다.",
            name
        )));
    }
    Ok(())
}
