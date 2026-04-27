pub mod gitignore;
pub mod license;
pub mod readme;

use crate::error::{GitBoostError, Result};
use crate::github::GithubClient;
use std::path::Path;

/// 파일 생성 요청 파라미터
pub struct WriteFilesArgs<'a> {
    pub project_name: &'a str,
    pub license_id: &'a str,
    pub author: &'a str,
    pub template: Option<&'a str>,
    pub dir: &'a Path,
    pub github_client: Option<&'a GithubClient>,
}

/// 프로젝트 디렉토리에 초기 파일들을 생성합니다.
///
/// - README.md: 이미 존재하면 스킵
/// - LICENSE: license_id != "none"이면 생성. 이미 존재하면 스킵
/// - .gitignore: template이 Some이면 GitHub API에서 다운로드. 이미 존재하면 스킵
pub async fn write_files(args: WriteFilesArgs<'_>) -> Result<()> {
    // README.md
    let readme_path = args.dir.join("README.md");
    if readme_path.exists() {
        crate::ui::warn("README.md가 이미 존재합니다. 덮어쓰지 않습니다.");
    } else {
        let content = readme::generate(args.project_name);
        std::fs::write(&readme_path, content)
            .map_err(|e| GitBoostError::Fs(format!("README.md 작성 실패: {}", e)))?;
    }

    // LICENSE
    if args.license_id.to_lowercase() != "none" {
        let license_path = args.dir.join("LICENSE");
        if license_path.exists() {
            crate::ui::warn("LICENSE가 이미 존재합니다. 덮어쓰지 않습니다.");
        } else if let Some(content) = license::generate(args.license_id, args.author) {
            std::fs::write(&license_path, content)
                .map_err(|e| GitBoostError::Fs(format!("LICENSE 작성 실패: {}", e)))?;
        }
    }

    // .gitignore
    if let Some(template) = args.template {
        let gitignore_path = args.dir.join(".gitignore");
        if gitignore_path.exists() {
            crate::ui::warn(".gitignore가 이미 존재합니다. 덮어쓰지 않습니다.");
        } else if let Some(client) = args.github_client {
            if let Some(content) = gitignore::fetch(client, template).await {
                std::fs::write(&gitignore_path, content)
                    .map_err(|e| GitBoostError::Fs(format!(".gitignore 작성 실패: {}", e)))?;
            }
        }
    }

    Ok(())
}
