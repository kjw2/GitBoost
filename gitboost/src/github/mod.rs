pub mod models;
pub mod repos;

use crate::error::{GitBoostError, Result};
pub use models::{CreateRepoRequest, GitignoreTemplate, Repo, User};

/// GitHub API 클라이언트
///
/// 모든 요청에 인증 헤더와 공통 헤더를 자동으로 첨부합니다.
pub struct GithubClient {
    client: reqwest::Client,
    token: String,
    api_base: String,
}

impl GithubClient {
    /// 새 GithubClient를 생성합니다.
    pub fn new(client: reqwest::Client, token: String) -> Self {
        Self {
            client,
            token,
            api_base: crate::config::github_api_base(),
        }
    }

    /// 인증 헤더가 포함된 GET 요청 빌더를 반환합니다.
    pub(crate) fn get(&self, path: &str) -> reqwest::RequestBuilder {
        self.client
            .get(format!("{}{}", self.api_base, path))
            .bearer_auth(&self.token)
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", crate::config::USER_AGENT)
            .header("X-GitHub-Api-Version", "2022-11-28")
    }

    /// 인증 헤더가 포함된 POST 요청 빌더를 반환합니다.
    pub(crate) fn post(&self, path: &str) -> reqwest::RequestBuilder {
        self.client
            .post(format!("{}{}", self.api_base, path))
            .bearer_auth(&self.token)
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", crate::config::USER_AGENT)
            .header("X-GitHub-Api-Version", "2022-11-28")
    }

    /// 인증 헤더가 포함된 DELETE 요청 빌더를 반환합니다.
    pub(crate) fn delete(&self, path: &str) -> reqwest::RequestBuilder {
        self.client
            .delete(format!("{}{}", self.api_base, path))
            .bearer_auth(&self.token)
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", crate::config::USER_AGENT)
            .header("X-GitHub-Api-Version", "2022-11-28")
    }

    /// GitHub API 응답을 처리합니다. 4xx/5xx는 GitHubError로 변환합니다.
    /// 401 Unauthorized 수신 시 keyring 토큰을 삭제하고 Auth 에러를 반환합니다.
    pub(crate) async fn handle_response<T: serde::de::DeserializeOwned>(
        resp: reqwest::Response,
    ) -> Result<T> {
        let status = resp.status();
        if status.is_success() {
            resp.json::<T>()
                .await
                .map_err(|e| GitBoostError::Network(format!("응답 파싱 실패: {}", e)))
        } else if status == reqwest::StatusCode::UNAUTHORIZED {
            // 토큰 만료/해지 — keyring 정리 후 재로그인 안내
            let _ = crate::auth::keyring_storage::delete();
            Err(GitBoostError::Auth(
                "GitHub 토큰이 만료되었거나 해지되었습니다.\n  \
                 `gitboost login`을 실행하여 다시 인증하세요."
                    .to_string(),
            ))
        } else {
            let status_code = status.as_u16();
            let message = resp
                .json::<models::ApiErrorBody>()
                .await
                .map(|b| b.message)
                .unwrap_or_else(|_| status.to_string());
            Err(GitBoostError::GitHub {
                status: status_code,
                message,
            })
        }
    }

    /// GitHub API 응답에서 body를 버립니다 (204 No Content 등).
    /// 401 Unauthorized 수신 시 keyring 토큰을 삭제하고 Auth 에러를 반환합니다.
    pub(crate) async fn handle_empty_response(resp: reqwest::Response) -> Result<()> {
        let status = resp.status();
        if status.is_success() {
            Ok(())
        } else if status == reqwest::StatusCode::UNAUTHORIZED {
            let _ = crate::auth::keyring_storage::delete();
            Err(GitBoostError::Auth(
                "GitHub 토큰이 만료되었거나 해지되었습니다.\n  \
                 `gitboost login`을 실행하여 다시 인증하세요."
                    .to_string(),
            ))
        } else {
            let status_code = status.as_u16();
            let message = resp
                .json::<models::ApiErrorBody>()
                .await
                .map(|b| b.message)
                .unwrap_or_else(|_| status.to_string());
            Err(GitBoostError::GitHub {
                status: status_code,
                message,
            })
        }
    }
}
