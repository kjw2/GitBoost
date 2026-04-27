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
/// Level 1: gh CLI 위임 (Zero-Config)
/// Level 2: OS keyring 조회 (Returning User)
/// Level 3: GitHub Device Flow (First-Time User)
pub async fn resolve_token(client: &reqwest::Client) -> Result<TokenSource> {
    // Level 1: gh CLI
    if let Some(token) = gh_cli::get_token().await {
        if verify_token(client, &token).await {
            tracing::debug!("인증 성공: gh CLI 위임");
            return Ok(TokenSource {
                token,
                origin: Origin::GhCli,
            });
        }
        tracing::debug!("gh CLI 토큰 검증 실패, 다음 단계로 진행");
    }

    // Level 2: keyring
    match keyring_storage::load() {
        Ok(Some(token)) => {
            if verify_token(client, &token).await {
                tracing::debug!("인증 성공: keyring");
                return Ok(TokenSource {
                    token,
                    origin: Origin::Keyring,
                });
            }
            // 만료된 토큰 keyring에서 삭제
            tracing::debug!("keyring 토큰 검증 실패, 삭제 후 다음 단계로 진행");
            let _ = keyring_storage::delete();
        }
        Ok(None) => tracing::debug!("keyring에 토큰 없음"),
        Err(e) => tracing::debug!("keyring 로드 실패: {}", e),
    }

    // Level 3: Device Flow
    tracing::debug!("Device Flow 인증 시작");
    let token = device_flow::run_device_flow(client).await?;
    keyring_storage::save(&token)?;
    Ok(TokenSource {
        token,
        origin: Origin::DeviceFlow,
    })
}

/// GitHub API /user 엔드포인트로 토큰 유효성을 검증합니다.
async fn verify_token(client: &reqwest::Client, token: &str) -> bool {
    let api_base = crate::config::github_api_base();
    let url = format!("{}/user", api_base);
    match client
        .get(&url)
        .bearer_auth(token)
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", crate::config::USER_AGENT)
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send()
        .await
    {
        Ok(resp) => resp.status().is_success(),
        Err(_) => false,
    }
}
