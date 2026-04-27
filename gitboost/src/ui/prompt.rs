use crate::error::Result;

/// 사용자에게 확인을 요청하는 프롬프트
///
/// `yes_flag`가 true이거나 stdin이 TTY가 아니면 `default`를 반환합니다.
pub fn confirm(question: &str, default: bool, yes_flag: bool) -> Result<bool> {
    if yes_flag {
        return Ok(default);
    }

    use std::io::IsTerminal;
    if !std::io::stdin().is_terminal() {
        return Ok(default);
    }

    let result = dialoguer::Confirm::new()
        .with_prompt(question)
        .default(default)
        .interact()
        .unwrap_or(default);

    Ok(result)
}
