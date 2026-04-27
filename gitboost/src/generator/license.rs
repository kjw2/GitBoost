use crate::error::{GitBoostError, Result};
use chrono::Datelike;

/// 바이너리에 임베드된 라이선스 (컴파일 타임)
const SUPPORTED_LICENSES: &[&str] = &[
    "mit",
    "apache-2.0",
    "bsd-3-clause",
    "gpl-3.0",
    "mpl-2.0",
    "unlicense",
    "none",
];

/// 바이너리에 직접 임베드된 라이선스 ID 목록
const EMBEDDED_LICENSES: &[&str] = &["mit", "apache-2.0", "bsd-3-clause"];

/// 라이선스 다운로드 URL (임베드 미포함 라이선스용)
fn download_url(id: &str) -> Option<&'static str> {
    match id {
        "gpl-3.0" => Some("https://www.gnu.org/licenses/gpl-3.0.txt"),
        "mpl-2.0" => Some("https://www.mozilla.org/media/MPL/2.0/index.txt"),
        "unlicense" => Some("https://unlicense.org/UNLICENSE"),
        _ => None,
    }
}

/// 라이선스 SPDX ID를 검증합니다.
pub fn validate(id: &str) -> Result<()> {
    let normalized = id.to_lowercase();
    if SUPPORTED_LICENSES.contains(&normalized.as_str()) {
        Ok(())
    } else {
        Err(GitBoostError::Prerequisite(format!(
            "지원하지 않는 라이선스 ID: '{}'\n  지원 목록: {}",
            id,
            SUPPORTED_LICENSES.join(", ")
        )))
    }
}

/// 라이선스 파일 내용을 생성합니다.
///
/// - `license_id`가 "none"이면 None을 반환합니다.
/// - MIT / Apache-2.0 / BSD-3-Clause는 바이너리에 임베드된 본문을 사용합니다.
/// - GPL-3.0 / MPL-2.0 / Unlicense는 다운로드 URL이 담긴 안내 파일을 생성합니다.
pub fn generate(license_id: &str, author: &str) -> Option<String> {
    let year = chrono::Local::now().year().to_string();
    let normalized = license_id.to_lowercase();

    // 임베드된 라이선스
    let embedded = match normalized.as_str() {
        "mit" => Some(include_str!("templates/mit.txt")),
        "apache-2.0" => Some(include_str!("templates/apache-2.0.txt")),
        "bsd-3-clause" => Some(include_str!("templates/bsd-3-clause.txt")),
        _ => None,
    };

    if let Some(template) = embedded {
        let content = template
            .replace("{{YEAR}}", &year)
            .replace("{{AUTHOR}}", author);
        return Some(content);
    }

    // 임베드 미포함 라이선스: 다운로드 안내 파일 생성
    if let Some(url) = download_url(&normalized) {
        crate::ui::warn(&format!(
            "{} 라이선스 본문은 바이너리에 포함되지 않습니다.",
            license_id.to_uppercase()
        ));
        crate::ui::warn(&format!(
            "  LICENSE 파일에 다운로드 URL을 기재합니다: {}",
            url
        ));
        crate::ui::warn("  이후 직접 파일을 해당 URL의 내용으로 교체하세요.");
        let placeholder = format!(
            "# {} License\n\
             #\n\
             # 이 파일은 자동 생성된 안내 파일입니다.\n\
             # 아래 URL에서 라이선스 전문을 내려받아 이 파일의 내용을 교체하세요:\n\
             #\n\
             #   {}\n\
             #\n\
             # Copyright (c) {} {}\n",
            license_id.to_uppercase(),
            url,
            year,
            author
        );
        return Some(placeholder);
    }

    // "none"
    None
}

/// 해당 라이선스가 바이너리에 임베드되어 있는지 반환합니다.
pub fn is_embedded(id: &str) -> bool {
    EMBEDDED_LICENSES.contains(&id.to_lowercase().as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_valid_licenses() {
        assert!(validate("mit").is_ok());
        assert!(validate("MIT").is_ok());
        assert!(validate("apache-2.0").is_ok());
        assert!(validate("none").is_ok());
        assert!(validate("unlicense").is_ok());
        assert!(validate("gpl-3.0").is_ok());
        assert!(validate("mpl-2.0").is_ok());
    }

    #[test]
    fn validate_invalid_license() {
        assert!(validate("copyleft").is_err());
        assert!(validate("").is_err());
    }

    #[test]
    fn generate_mit_replaces_placeholders() {
        let content = generate("mit", "Alice").unwrap();
        let year = chrono::Local::now().year().to_string();
        assert!(content.contains(&year));
        assert!(content.contains("Alice"));
        assert!(!content.contains("{{YEAR}}"));
        assert!(!content.contains("{{AUTHOR}}"));
    }

    #[test]
    fn generate_bsd_replaces_placeholders() {
        let content = generate("bsd-3-clause", "Bob").unwrap();
        assert!(content.contains("Bob"));
        assert!(!content.contains("{{AUTHOR}}"));
    }

    #[test]
    fn generate_none_returns_none() {
        assert!(generate("none", "anyone").is_none());
    }

    #[test]
    fn generate_apache_no_placeholders_to_replace() {
        let content = generate("apache-2.0", "Carol").unwrap();
        assert!(!content.is_empty());
    }

    #[test]
    fn generate_non_embedded_returns_placeholder() {
        // gpl-3.0, mpl-2.0, unlicense는 다운로드 안내 파일을 생성
        let content = generate("gpl-3.0", "Dave").unwrap();
        assert!(content.contains("gpl-3.0.txt") || content.contains("gnu.org"));

        let content2 = generate("mpl-2.0", "Dave").unwrap();
        assert!(content2.contains("mozilla.org"));

        let content3 = generate("unlicense", "Dave").unwrap();
        assert!(content3.contains("unlicense.org"));
    }

    #[test]
    fn embedded_check() {
        assert!(is_embedded("mit"));
        assert!(is_embedded("apache-2.0"));
        assert!(is_embedded("bsd-3-clause"));
        assert!(!is_embedded("gpl-3.0"));
        assert!(!is_embedded("mpl-2.0"));
        assert!(!is_embedded("unlicense"));
    }
}
