use serde::{Deserialize, Serialize};

/// GitHub 사용자 정보
#[derive(Deserialize, Debug)]
pub struct User {
    pub login: String,
    pub name: Option<String>,
    pub email: Option<String>,
}

/// GitHub 저장소 정보
#[derive(Deserialize, Debug)]
pub struct Repo {
    pub name: String,
    pub full_name: String,
    pub html_url: String,
    pub clone_url: String,
    pub ssh_url: String,
    pub private: bool,
}

/// 저장소 생성 요청 바디
#[derive(Serialize, Debug)]
pub struct CreateRepoRequest<'a> {
    pub name: &'a str,
    pub private: bool,
    pub description: Option<&'a str>,
    pub auto_init: bool,
}

/// .gitignore 템플릿 응답
#[derive(Deserialize, Debug)]
pub struct GitignoreTemplate {
    pub name: String,
    pub source: String,
}

/// GitHub API 에러 응답 바디
#[derive(Deserialize, Debug)]
pub struct ApiErrorBody {
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_user() {
        let json = r#"{"login":"alice","name":"Alice","email":"alice@example.com"}"#;
        let user: User = serde_json::from_str(json).unwrap();
        assert_eq!(user.login, "alice");
        assert_eq!(user.name.as_deref(), Some("Alice"));
    }

    #[test]
    fn deserialize_user_no_email() {
        let json = r#"{"login":"bob","name":null,"email":null}"#;
        let user: User = serde_json::from_str(json).unwrap();
        assert_eq!(user.login, "bob");
        assert!(user.name.is_none());
        assert!(user.email.is_none());
    }

    #[test]
    fn deserialize_repo() {
        let json = r#"{
            "name":"my-repo","full_name":"alice/my-repo",
            "html_url":"https://github.com/alice/my-repo",
            "clone_url":"https://github.com/alice/my-repo.git",
            "ssh_url":"git@github.com:alice/my-repo.git",
            "private":true
        }"#;
        let repo: Repo = serde_json::from_str(json).unwrap();
        assert_eq!(repo.name, "my-repo");
        assert!(repo.private);
    }

    #[test]
    fn deserialize_api_error() {
        let json = r#"{"message":"Repository creation failed."}"#;
        let err: ApiErrorBody = serde_json::from_str(json).unwrap();
        assert_eq!(err.message, "Repository creation failed.");
    }
}
