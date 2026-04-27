use thiserror::Error;

/// GitBoost의 모든 에러 타입
#[derive(Error, Debug)]
pub enum GitBoostError {
    /// git, gh 등 사전 요구사항 미충족
    #[error("사전 요구사항 미충족: {0}")]
    Prerequisite(String),

    /// GitHub 인증 실패
    #[error("인증 실패: {0}")]
    Auth(String),

    /// GitHub API 에러 응답
    #[error("GitHub API 에러 ({status}): {message}")]
    GitHub { status: u16, message: String },

    /// 로컬 파일시스템 에러
    #[error("로컬 파일시스템 에러: {0}")]
    Fs(String),

    /// git 명령 실행 실패
    #[error("Git 명령 실패: {cmd}\n  stderr: {stderr}")]
    Git { cmd: String, stderr: String },

    /// 네트워크 에러
    #[error("네트워크 에러: {0}")]
    Network(String),

    /// 사용자가 작업을 취소
    #[error("사용자가 작업을 취소했습니다")]
    UserAborted,

    /// 기타 에러 (anyhow 래핑)
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl GitBoostError {
    /// 에러 종류에 따른 종료 코드 반환
    pub fn exit_code(&self) -> i32 {
        match self {
            GitBoostError::Prerequisite(_) => 10,
            GitBoostError::Auth(_) => 11,
            GitBoostError::GitHub { .. } => 12,
            GitBoostError::Fs(_) => 13,
            GitBoostError::Git { .. } => 14,
            GitBoostError::Network(_) => 15,
            GitBoostError::UserAborted => 1,
            GitBoostError::Other(_) => 1,
        }
    }

    /// 에러 발생 후 사용자가 취할 수 있는 행동 힌트
    pub fn next_action_hint(&self) -> Option<&'static str> {
        match self {
            GitBoostError::Prerequisite(_) => {
                Some("git 2.28+ 가 설치되어 있는지 확인하세요: https://git-scm.com/downloads")
            }
            GitBoostError::Auth(_) => Some("다시 인증하려면 `gitboost login`을 실행하세요."),
            GitBoostError::GitHub { status: 401, .. }
            | GitBoostError::GitHub { status: 403, .. } => {
                Some("토큰 권한을 확인하거나 `gitboost login`으로 재인증하세요.")
            }
            GitBoostError::GitHub { status: 422, .. } => {
                Some("저장소 이름이 이미 사용 중이거나 유효하지 않습니다. 다른 이름을 사용하세요.")
            }
            GitBoostError::Git { .. } => {
                Some("git 설정을 확인하세요 (`git config user.name`, `git config user.email`).")
            }
            GitBoostError::Network(_) => Some("인터넷 연결을 확인하고 다시 시도하세요."),
            _ => None,
        }
    }
}

/// GitBoost 전용 Result 타입
pub type Result<T> = std::result::Result<T, GitBoostError>;
