/// README.md 내용을 생성합니다.
///
/// - `description`이 있으면 제목 아래에 삽입합니다.
/// - 설치 / 사용법 / 기여 / 라이선스 섹션을 포함합니다.
pub fn generate(project_name: &str, description: Option<&str>, license_id: &str) -> String {
    let desc_section = match description {
        Some(d) if !d.is_empty() => format!("\n{}\n", d),
        _ => String::new(),
    };

    let license_badge = match license_id.to_lowercase().as_str() {
        "mit" => "[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)",
        "apache-2.0" => "[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)",
        "gpl-3.0" => "[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](LICENSE)",
        "bsd-3-clause" => "[![License](https://img.shields.io/badge/License-BSD%203--Clause-blue.svg)](LICENSE)",
        "mpl-2.0" => "[![License: MPL 2.0](https://img.shields.io/badge/License-MPL%202.0-brightgreen.svg)](LICENSE)",
        "unlicense" => "[![License: Unlicense](https://img.shields.io/badge/license-Unlicense-blue.svg)](LICENSE)",
        _ => "",
    };

    let badge_line = if license_badge.is_empty() {
        String::new()
    } else {
        format!("\n{}\n", license_badge)
    };

    let license_section = if license_id.to_lowercase() == "none" {
        String::new()
    } else {
        format!(
            "\n## License\n\nThis project is licensed under the {} License — see the [LICENSE](LICENSE) file for details.\n",
            match license_id.to_lowercase().as_str() {
                "mit" => "MIT",
                "apache-2.0" => "Apache 2.0",
                "gpl-3.0" => "GNU General Public v3",
                "bsd-3-clause" => "BSD 3-Clause",
                "mpl-2.0" => "Mozilla Public 2.0",
                "unlicense" => "Unlicense",
                other => other,
            }
        )
    };

    format!(
        "# {name}{badge}{desc}\n\
         ## Getting Started\n\
         \n\
         ```bash\n\
         # clone\n\
         git clone https://github.com/<your-username>/{name}.git\n\
         cd {name}\n\
         ```\n\
         \n\
         ## Usage\n\
         \n\
         > Add usage instructions here.\n\
         \n\
         ## Contributing\n\
         \n\
         Pull requests are welcome. For major changes, please open an issue first.\n\
         {license}",
        name = project_name,
        badge = badge_line,
        desc = desc_section,
        license = license_section,
    )
}
