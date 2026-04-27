use crate::error::{GitBoostError, Result};

/// git 및 gh CLI의 사전 요구사항 검사 결과
pub struct PrereqResult {
    /// gh CLI 설치 여부
    pub gh_available: bool,
}

/// 사전 요구사항을 검사합니다.
///
/// - git 2.28+ 필수
/// - gh CLI 선택 (없어도 계속 진행)
pub fn check() -> Result<PrereqResult> {
    check_git()?;
    let gh_available = check_gh();
    Ok(PrereqResult { gh_available })
}

/// git 버전을 확인하고 2.28 미만이면 에러를 반환합니다
fn check_git() -> Result<()> {
    let output = std::process::Command::new("git")
        .arg("--version")
        .output()
        .map_err(|_| {
            GitBoostError::Prerequisite(
                "git이 설치되어 있지 않습니다. https://git-scm.com/downloads 에서 설치하세요."
                    .to_string(),
            )
        })?;

    if !output.status.success() {
        return Err(GitBoostError::Prerequisite(
            "git 버전 확인에 실패했습니다.".to_string(),
        ));
    }

    let version_str = String::from_utf8_lossy(&output.stdout);
    parse_and_validate_git_version(&version_str)
}

/// git 버전 문자열을 파싱하여 최소 버전(2.28) 요건 확인
fn parse_and_validate_git_version(version_str: &str) -> Result<()> {
    // 예시: "git version 2.43.0"
    let parts: Vec<&str> = version_str.split_whitespace().collect();
    if parts.len() < 3 {
        return Err(GitBoostError::Prerequisite(format!(
            "git 버전을 파싱할 수 없습니다: {}",
            version_str.trim()
        )));
    }

    let version_part = parts[2];
    // Windows에서는 "2.43.0.windows.1" 형태일 수 있으므로 첫 세그먼트만 사용
    let semver_part = version_part.split('.').collect::<Vec<_>>();

    if semver_part.len() < 2 {
        return Err(GitBoostError::Prerequisite(format!(
            "git 버전 형식을 인식할 수 없습니다: {}",
            version_part
        )));
    }

    let major: u32 = semver_part[0].parse().unwrap_or(0);
    let minor: u32 = semver_part[1].parse().unwrap_or(0);

    if major < 2 || (major == 2 && minor < 28) {
        return Err(GitBoostError::Prerequisite(format!(
            "git 2.28 이상이 필요합니다. 현재 버전: {}. \
             https://git-scm.com/downloads 에서 업데이트하세요.",
            version_part
        )));
    }

    Ok(())
}

/// gh CLI 설치 여부 확인
fn check_gh() -> bool {
    std::process::Command::new("gh")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn git_version_valid() {
        assert!(parse_and_validate_git_version("git version 2.43.0").is_ok());
        assert!(parse_and_validate_git_version("git version 2.28.0").is_ok());
        assert!(parse_and_validate_git_version("git version 3.0.0").is_ok());
    }

    #[test]
    fn git_version_too_old() {
        assert!(parse_and_validate_git_version("git version 2.27.0").is_err());
        assert!(parse_and_validate_git_version("git version 1.9.0").is_err());
    }
}
