use crate::error::{GitBoostError, Result};
use std::path::{Path, PathBuf};

/// RAII guard: `Drop` 시 remote URL을 원복합니다.
struct RestoreRemoteUrl {
    cwd: PathBuf,
    remote: String,
    original_url: String,
}

impl Drop for RestoreRemoteUrl {
    fn drop(&mut self) {
        let _ = std::process::Command::new("git")
            .args(["remote", "set-url", &self.remote, &self.original_url])
            .current_dir(&self.cwd)
            .output();
    }
}

/// 토큰을 HTTPS URL에 임시 주입하여 git push를 실행합니다.
///
/// credential helper 대신 `git remote set-url`로 `x-access-token:{token}@`를 삽입한 뒤
/// push 완료 후 RAII guard가 원래 URL로 원복합니다.
/// Unix / Windows 공통 코드 경로를 사용하므로 bat 파일이나 셸 스크립트가 필요 없습니다.
pub fn push_with_token(cwd: &Path, token: &str, remote: &str, branch: &str) -> Result<()> {
    use std::process::Stdio;

    // 1. 현재 remote URL 조회
    let get_url_output = std::process::Command::new("git")
        .args(["remote", "get-url", remote])
        .current_dir(cwd)
        .output()
        .map_err(|e| GitBoostError::Git {
            cmd: format!("git remote get-url {}", remote),
            stderr: e.to_string(),
        })?;

    let original_url = String::from_utf8_lossy(&get_url_output.stdout)
        .trim()
        .to_string();

    // 2. 토큰을 URL에 주입 (https://github.com/... → https://x-access-token:{token}@github.com/...)
    let auth_url = build_auth_url(&original_url, token)?;

    // 3. 인증 URL로 임시 변경
    let set_url_output = std::process::Command::new("git")
        .args(["remote", "set-url", remote, &auth_url])
        .current_dir(cwd)
        .output()
        .map_err(|e| GitBoostError::Git {
            cmd: "git remote set-url".to_string(),
            stderr: e.to_string(),
        })?;

    if !set_url_output.status.success() {
        return Err(GitBoostError::Git {
            cmd: "git remote set-url".to_string(),
            stderr: String::from_utf8_lossy(&set_url_output.stderr)
                .trim()
                .to_string(),
        });
    }

    // 4. RAII guard 등록 — push 성공/실패 모두 원복 보장
    let _restore = RestoreRemoteUrl {
        cwd: cwd.to_path_buf(),
        remote: remote.to_string(),
        original_url,
    };

    // 5. push 실행
    let output = std::process::Command::new("git")
        .args(["push", "-u", remote, branch])
        .current_dir(cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| GitBoostError::Git {
            cmd: "git push".to_string(),
            stderr: e.to_string(),
        })?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(GitBoostError::Git {
            cmd: format!("git push -u {} {}", remote, branch),
            stderr,
        })
    }
}

/// `https://github.com/owner/repo` → `https://x-access-token:{token}@github.com/owner/repo`
fn build_auth_url(url: &str, token: &str) -> Result<String> {
    if let Some(rest) = url.strip_prefix("https://") {
        Ok(format!("https://x-access-token:{}@{}", token, rest))
    } else {
        Err(GitBoostError::Auth(format!(
            "지원하지 않는 remote URL 형식입니다: {}\n  HTTPS URL만 지원됩니다.",
            url
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::build_auth_url;

    #[test]
    fn test_build_auth_url() {
        let url = "https://github.com/kjw2/testProject";
        let result = build_auth_url(url, "mytoken").unwrap();
        assert_eq!(
            result,
            "https://x-access-token:mytoken@github.com/kjw2/testProject"
        );
    }

    #[test]
    fn test_build_auth_url_rejects_ssh() {
        let url = "git@github.com:kjw2/testProject.git";
        assert!(build_auth_url(url, "mytoken").is_err());
    }
}
