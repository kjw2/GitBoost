use crate::error::{GitBoostError, Result};
use serde::Deserialize;

#[derive(Deserialize)]
struct DeviceCodeResponse {
    device_code: String,
    user_code: String,
    verification_uri: String,
    expires_in: u64,
    interval: u64,
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: Option<String>,
    error: Option<String>,
}

/// GitHub Device Flow를 실행하여 액세스 토큰을 발급받습니다.
///
/// Ctrl+C 시 UserAborted 에러를 반환합니다.
pub async fn run_device_flow(client: &reqwest::Client) -> Result<String> {
    let client_id = crate::config::GITHUB_CLIENT_ID;
    if client_id.is_empty() {
        return Err(GitBoostError::Auth(
            "GitHub OAuth App Client ID가 설정되지 않았습니다.\n  \
             GITBOOST_GITHUB_CLIENT_ID 환경변수를 설정하거나 gh CLI를 사용하세요."
                .to_string(),
        ));
    }

    let oauth_base = crate::config::GITHUB_OAUTH_BASE;

    // 1. device code 요청
    let device_resp = client
        .post(format!("{}/login/device/code", oauth_base))
        .header("Accept", "application/json")
        .header("User-Agent", crate::config::USER_AGENT)
        .form(&[("client_id", client_id), ("scope", "repo")])
        .send()
        .await
        .map_err(|e| GitBoostError::Network(e.to_string()))?;

    if !device_resp.status().is_success() {
        return Err(GitBoostError::Auth(format!(
            "device code 요청 실패: HTTP {}",
            device_resp.status()
        )));
    }

    let device_data: DeviceCodeResponse = device_resp
        .json()
        .await
        .map_err(|e| GitBoostError::Network(format!("device code 응답 파싱 실패: {}", e)))?;

    // 2. 사용자에게 안내
    crate::ui::info("\n  브라우저에서 다음 URL을 열고 코드를 입력하세요:");
    crate::ui::info(&format!("    URL : {}", device_data.verification_uri));
    crate::ui::info(&format!("    CODE: {}\n", device_data.user_code));

    // 브라우저 자동 오픈 시도 (실패해도 계속 진행)
    let _ = webbrowser::open(&device_data.verification_uri);

    // 3. 폴링
    let mut interval = device_data.interval;
    let deadline =
        tokio::time::Instant::now() + tokio::time::Duration::from_secs(device_data.expires_in);

    loop {
        // Ctrl+C 또는 timeout 대기
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                return Err(GitBoostError::UserAborted);
            }
            _ = tokio::time::sleep(tokio::time::Duration::from_secs(interval)) => {}
        }

        if tokio::time::Instant::now() >= deadline {
            return Err(GitBoostError::Auth(
                "인증 시간이 만료되었습니다 (expired_token).\n  \
                 다시 시도하려면 `gitboost login`을 실행하세요."
                    .to_string(),
            ));
        }

        // token 폴링 요청
        let token_resp = client
            .post(format!("{}/login/oauth/access_token", oauth_base))
            .header("Accept", "application/json")
            .header("User-Agent", crate::config::USER_AGENT)
            .form(&[
                ("client_id", client_id),
                ("device_code", device_data.device_code.as_str()),
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
            ])
            .send()
            .await
            .map_err(|e| GitBoostError::Network(e.to_string()))?;

        let token_data: TokenResponse = token_resp
            .json()
            .await
            .map_err(|e| GitBoostError::Network(format!("token 응답 파싱 실패: {}", e)))?;

        if let Some(token) = token_data.access_token {
            if !token.is_empty() {
                tracing::debug!("Device Flow 인증 성공");
                return Ok(token);
            }
        }

        match token_data.error.as_deref() {
            Some("authorization_pending") => {
                tracing::debug!("인증 대기 중...");
                continue;
            }
            Some("slow_down") => {
                interval += 5;
                tracing::debug!("slow_down 응답, interval을 {}초로 증가", interval);
                continue;
            }
            Some("expired_token") => {
                return Err(GitBoostError::Auth(
                    "인증 코드가 만료되었습니다.\n  \
                     다시 시도하려면 `gitboost login`을 실행하세요."
                        .to_string(),
                ));
            }
            Some("access_denied") => {
                return Err(GitBoostError::Auth(
                    "사용자가 인증을 거부했습니다.".to_string(),
                ));
            }
            Some(other) => {
                return Err(GitBoostError::Auth(format!("Device Flow 오류: {}", other)));
            }
            None => {
                return Err(GitBoostError::Auth(
                    "Device Flow: 알 수 없는 응답을 받았습니다.".to_string(),
                ));
            }
        }
    }
}
