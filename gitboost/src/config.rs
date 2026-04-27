/// GitBoost 앱 전역 상수 및 설정
pub const APP_NAME: &str = "gitboost";

/// 현재 버전 (Cargo.toml에서 가져옴)
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// HTTP User-Agent 헤더 값
pub const USER_AGENT: &str = concat!("gitboost/", env!("CARGO_PKG_VERSION"));

/// keyring 서비스 이름
pub const KEYRING_SERVICE: &str = "gitboost";

/// keyring 사용자 이름
pub const KEYRING_USER: &str = "github_token";

/// 컴파일타임에 주입된 GitHub OAuth App Client ID
pub const GITHUB_CLIENT_ID: &str = env!("GITBOOST_GITHUB_CLIENT_ID");

/// GitHub API 기본 URL
pub const GITHUB_API_BASE_DEFAULT: &str = "https://api.github.com";

/// GitHub OAuth 기본 URL
pub const GITHUB_OAUTH_BASE: &str = "https://github.com";

/// 런타임에 GITHUB_API_BASE 환경변수를 확인하여 테스트용 URL로 오버라이드 가능
pub fn github_api_base() -> String {
    std::env::var("GITHUB_API_BASE").unwrap_or_else(|_| GITHUB_API_BASE_DEFAULT.to_string())
}
