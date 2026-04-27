use crate::error::Result;
use crate::github::GithubClient;

/// 사용자 입력값을 GitHub API gitignore 템플릿 이름으로 정규화합니다.
pub fn normalize_template_name(input: &str) -> &str {
    match input.to_lowercase().as_str() {
        "rust" => "Rust",
        "node" | "nodejs" | "node.js" => "Node",
        "python" => "Python",
        "go" | "golang" => "Go",
        "java" => "Java",
        "cpp" | "c++" => "C++",
        "c" => "C",
        "csharp" | "cs" | "c#" => "CSharp",
        "ruby" => "Ruby",
        "php" => "PHP",
        "swift" => "Swift",
        "kotlin" => "Kotlin",
        "scala" => "Scala",
        "elixir" => "Elixir",
        "haskell" => "Haskell",
        "dart" => "Dart",
        "lua" => "Lua",
        "perl" => "Perl",
        "unity" => "Unity",
        "unreal" | "unrealengine" => "UnrealEngine",
        "godot" => "Godot",
        "android" => "Android",
        // 매핑 없으면 입력값 그대로 (사용자가 정확한 이름을 안다고 가정)
        _ => input,
    }
}

/// GitHub API에서 .gitignore 템플릿을 다운로드합니다.
///
/// 실패 시 None을 반환하고 경고를 출력합니다 (치명적 오류 아님).
pub async fn fetch(client: &GithubClient, template: &str) -> Option<String> {
    let name = normalize_template_name(template);
    match client.fetch_gitignore_template(name).await {
        Ok(t) => Some(t.source),
        Err(e) => {
            crate::ui::warn(&format!(
                ".gitignore 템플릿 '{}' 다운로드 실패: {}. 건너뜁니다.",
                name, e
            ));
            None
        }
    }
}

/// .gitignore 템플릿 이름 정규화가 GitHub API 형식과 일치하는지 확인합니다.
pub fn validate_template_name(_name: &str) -> Result<()> {
    // 현재 모든 입력값을 허용하고 실제 API 호출 시 검증
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_common_names() {
        assert_eq!(normalize_template_name("rust"), "Rust");
        assert_eq!(normalize_template_name("Rust"), "Rust");
        assert_eq!(normalize_template_name("node"), "Node");
        assert_eq!(normalize_template_name("python"), "Python");
        assert_eq!(normalize_template_name("go"), "Go");
        assert_eq!(normalize_template_name("java"), "Java");
    }

    #[test]
    fn normalize_aliases() {
        assert_eq!(normalize_template_name("nodejs"), "Node");
        assert_eq!(normalize_template_name("c++"), "C++");
        assert_eq!(normalize_template_name("csharp"), "CSharp");
    }

    #[test]
    fn normalize_unknown_passthrough() {
        // 매핑 없는 값은 입력값 그대로 반환
        assert_eq!(normalize_template_name("MyCustomLang"), "MyCustomLang");
    }
}
