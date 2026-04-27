use crate::error::{GitBoostError, Result};
use std::path::{Path, PathBuf};
use std::process::Stdio;

/// git 명령 실행 헬퍼
pub struct GitRunner {
    cwd: PathBuf,
}

impl GitRunner {
    /// 지정한 디렉토리를 작업 디렉토리로 사용하는 GitRunner를 생성합니다.
    pub fn new(cwd: impl Into<PathBuf>) -> Self {
        Self { cwd: cwd.into() }
    }

    /// git 명령을 실행하고 stdout을 반환합니다.
    pub fn run(&self, args: &[&str]) -> Result<String> {
        self.run_with_env(args, &[])
    }

    /// 환경변수와 함께 git 명령을 실행합니다.
    pub fn run_with_env(&self, args: &[&str], env: &[(&str, &str)]) -> Result<String> {
        tracing::debug!("git {}", args.join(" "));

        let mut cmd = std::process::Command::new("git");
        cmd.args(args)
            .current_dir(&self.cwd)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        for (key, val) in env {
            cmd.env(key, val);
        }

        let output = cmd.output().map_err(|e| GitBoostError::Git {
            cmd: format!("git {}", args.join(" ")),
            stderr: e.to_string(),
        })?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            Err(GitBoostError::Git {
                cmd: format!("git {}", args.join(" ")),
                stderr,
            })
        }
    }

    /// 현재 작업 디렉토리를 반환합니다.
    pub fn cwd(&self) -> &Path {
        &self.cwd
    }
}
