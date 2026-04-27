use super::{GithubClient, GitignoreTemplate, Repo, User};
use crate::error::{GitBoostError, Result};
use crate::github::models::CreateRepoRequest;

impl GithubClient {
    /// 현재 인증된 GitHub 사용자 정보를 조회합니다.
    pub async fn get_user(&self) -> Result<User> {
        let resp = self
            .get("/user")
            .send()
            .await
            .map_err(|e| GitBoostError::Network(e.to_string()))?;
        Self::handle_response(resp).await
    }

    /// 저장소 정보를 조회합니다. 존재하지 않으면 None을 반환합니다.
    pub async fn get_repo(&self, owner: &str, repo: &str) -> Result<Option<Repo>> {
        let resp = self
            .get(&format!("/repos/{}/{}", owner, repo))
            .send()
            .await
            .map_err(|e| GitBoostError::Network(e.to_string()))?;

        if resp.status() == 404 {
            return Ok(None);
        }
        Self::handle_response(resp).await.map(Some)
    }

    /// 새 GitHub 저장소를 생성합니다.
    pub async fn create_repo(&self, req: &CreateRepoRequest<'_>) -> Result<Repo> {
        let resp = self
            .post("/user/repos")
            .json(req)
            .send()
            .await
            .map_err(|e| GitBoostError::Network(e.to_string()))?;
        Self::handle_response(resp).await
    }

    /// 저장소를 삭제합니다 (롤백 용도).
    pub async fn delete_repo(&self, owner: &str, repo: &str) -> Result<()> {
        let resp = self
            .delete(&format!("/repos/{}/{}", owner, repo))
            .send()
            .await
            .map_err(|e| GitBoostError::Network(e.to_string()))?;
        Self::handle_empty_response(resp).await
    }

    /// .gitignore 템플릿을 GitHub API에서 가져옵니다.
    pub async fn fetch_gitignore_template(&self, name: &str) -> Result<GitignoreTemplate> {
        let resp = self
            .get(&format!("/gitignore/templates/{}", name))
            .send()
            .await
            .map_err(|e| GitBoostError::Network(e.to_string()))?;
        Self::handle_response(resp).await
    }
}
