pub mod device_flow;
pub mod gh_cli;
pub mod keyring_storage;

use crate::error::Result;

/// 토큰의 출처
#[derive(Debug, Clone, PartialEq)]
pub enum Origin {
    /// gh CLI에서 위임받은 토큰
    GhCli,
    /// OS keyring에 저장된 토큰
    Keyring,
    /// Device Flow로 새로 발급받은 토큰
    DeviceFlow,
}

/// 토큰과 출처를 함께 보관하는 구조체
pub struct TokenSource {
    pub token: String,
    pub origin: Origin,
}

/// GitHub 토큰을 3단계 우선순위로 해석합니다.
///
/// 네트워크 검증 없이 저장된 토큰을 즉시 반환합니다.
/// 토큰 유효성 확인은 실제 API 호출 시점에 자연스럽게 이루어지며,
/// 401 응답 수신 시 keyring을 자동으로 정리합니다.
///
/// Level 1: gh CLI 위임 (Zero-Config)
/// Level 2: OS keyring 조회 (Returning User)
/// Level 3: GitHub Device Flow (First-Time User)
pub async fn resolve_token(client: &reqwest::Client) -> Result<TokenSource> {
    // Level 1: gh CLI — gh가 토큰 갱신을 자체 관리하므로 그대로 신뢰
    if let Some(token) = gh_cli::get_token().await {
        tracing::debug!("토큰 출처: gh CLI");
        return Ok(TokenSource {
            token,
            origin: Origin::GhCli,
        });
    }

    // Level 2: keyring — 저장된 토큰을 네트워크 검증 없이 즉시 반환
    // 토큰이 만료된 경우 GitHub API가 401을 반환하며, handle_response에서 처리
    match keyring_storage::load() {
        Ok(Some(token)) => {
            tracing::debug!("토큰 출처: keyring");
            return Ok(TokenSource {
                token,
                origin: Origin::Keyring,
            });
        }
        Ok(None) => tracing::debug!("keyring에 토큰 없음"),
        Err(e) => tracing::debug!("keyring 로드 실패: {}", e),
    }

    // Level 3: Device Flow — 토큰 없을 때만 진입
    tracing::debug!("Device Flow 인증 시작");
    let token = device_flow::run_device_flow(client).await?;
    keyring_storage::save(&token)?;
    Ok(TokenSource {
        token,
        origin: Origin::DeviceFlow,
    })
}
