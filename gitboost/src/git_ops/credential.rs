use crate::error::{GitBoostError, Result};
use std::path::Path;

/// 토큰을 임시 credential helper로 주입하여 git push를 실행합니다.
///
/// Unix: 인라인 셸 함수로 credential helper 설정
/// Windows: 임시 .bat 파일 생성 후 사용, 종료 시 삭제
pub fn push_with_token(cwd: &Path, token: &str, remote: &str, branch: &str) -> Result<()> {
    #[cfg(not(windows))]
    {
        push_unix(cwd, token, remote, branch)
    }
    #[cfg(windows)]
    {
        push_windows(cwd, token, remote, branch)
    }
}

#[cfg(not(windows))]
fn push_unix(cwd: &Path, token: &str, remote: &str, branch: &str) -> Result<()> {
    use std::process::Stdio;

    // 인라인 셸 함수로 credential helper를 임시 설정
    // 토큰은 환경변수로 자식 프로세스에만 전달
    let helper_script =
        "!f() { echo \"username=x-access-token\"; echo \"password=$GITBOOST_TOKEN\"; }; f";

    let output = std::process::Command::new("git")
        .args([
            "-c",
            "credential.helper=",
            "-c",
            &format!("credential.helper={}", helper_script),
            "push",
            "-u",
            remote,
            branch,
        ])
        .current_dir(cwd)
        .env("GITBOOST_TOKEN", token)
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

#[cfg(windows)]
fn push_windows(cwd: &Path, token: &str, remote: &str, branch: &str) -> Result<()> {
    use std::io::Write;
    use std::process::Stdio;

    // 임시 .bat 파일 생성
    let pid = std::process::id();
    let bat_path = std::env::temp_dir().join(format!("gitboost-cred-{}.bat", pid));

    // RAII guard: 함수 종료 시 .bat 파일 삭제 보장
    struct TempFile(std::path::PathBuf);
    impl Drop for TempFile {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.0);
        }
    }
    let _guard = TempFile(bat_path.clone());

    {
        let mut file = std::fs::File::create(&bat_path)
            .map_err(|e| GitBoostError::Fs(format!("credential helper 파일 생성 실패: {}", e)))?;
        writeln!(file, "@echo off").ok();
        writeln!(file, "echo username=x-access-token").ok();
        writeln!(file, "echo password=%GITBOOST_TOKEN%").ok();
    }

    let bat_path_str = bat_path
        .to_str()
        .ok_or_else(|| GitBoostError::Fs("임시 파일 경로 변환 실패".to_string()))?;

    let output = std::process::Command::new("git")
        .args([
            "-c",
            "credential.helper=",
            "-c",
            &format!("credential.helper={}", bat_path_str),
            "push",
            "-u",
            remote,
            branch,
        ])
        .current_dir(cwd)
        .env("GITBOOST_TOKEN", token)
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
