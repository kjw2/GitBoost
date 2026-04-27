use clap::{Args, Parser, Subcommand};

/// GitBoost CLI: 프로젝트 이름 한 번으로 로컬 Git + GitHub 저장소를 자동 생성
#[derive(Parser, Debug)]
#[command(name = "gitboost", version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    /// 디버그 로그 출력 활성화
    #[arg(short, long, global = true)]
    pub verbose: bool,
}

/// 사용 가능한 서브커맨드
#[derive(Subcommand, Debug)]
pub enum Command {
    /// 새 프로젝트와 GitHub 저장소를 한 번에 생성합니다
    Create(CreateArgs),
    /// GitHub Device Flow로 명시적 로그인을 수행합니다
    Login,
    /// keyring에 저장된 GitBoost 토큰을 삭제합니다
    Logout,
    /// 현재 인증된 GitHub 사용자 정보를 출력합니다
    Whoami,
}

/// `create` 서브커맨드의 인자 정의
#[derive(Args, Debug)]
pub struct CreateArgs {
    /// 프로젝트(=저장소) 이름. 영문/숫자/하이픈/언더스코어/점만 허용 (최대 100자)
    pub name: String,

    /// Public 저장소로 생성 (기본은 Private)
    #[arg(long)]
    pub public: bool,

    /// SPDX 라이선스 ID (mit/apache-2.0/gpl-3.0/bsd-3-clause/mpl-2.0/unlicense/none)
    #[arg(short, long, default_value = "mit")]
    pub license: String,

    /// .gitignore 템플릿 언어 (rust/node/python/go/java 등)
    #[arg(short, long)]
    pub template: Option<String>,

    /// 저장소 설명
    #[arg(short, long)]
    pub description: Option<String>,

    /// 라이선스 저작자 이름 (기본: git config user.name → GitHub user.name)
    #[arg(long)]
    pub author: Option<String>,

    /// 원격 저장소는 만들되 첫 push는 생략
    #[arg(long)]
    pub no_push: bool,

    /// 모든 확인 프롬프트에 자동으로 동의 (CI 환경용)
    #[arg(short, long)]
    pub yes: bool,
}

/// 저장소 이름의 유효성을 검사하는 정규식 패턴
/// GitHub 저장소 이름 기준: 영문, 숫자, 하이픈, 언더스코어, 점, 최대 100자
pub fn validate_repo_name(name: &str) -> bool {
    // 정규식: ^[A-Za-z0-9._-]{1,100}$
    if name.is_empty() || name.len() > 100 {
        return false;
    }
    name.chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_repo_names() {
        assert!(validate_repo_name("my-project"));
        assert!(validate_repo_name("my_project"));
        assert!(validate_repo_name("MyProject123"));
        assert!(validate_repo_name("a"));
        assert!(validate_repo_name("my.project"));
        assert!(validate_repo_name(&"a".repeat(100)));
    }

    #[test]
    fn invalid_repo_names() {
        assert!(!validate_repo_name(""));
        assert!(!validate_repo_name("in valid"));
        assert!(!validate_repo_name("my/project"));
        assert!(!validate_repo_name("my:project"));
        assert!(!validate_repo_name(&"a".repeat(101)));
    }
}
