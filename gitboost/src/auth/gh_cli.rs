/// gh CLI에서 토큰을 가져옵니다.
///
/// gh가 없거나 토큰이 비어있으면 None을 반환합니다.
/// gh CLI 토큰은 keyring에 저장하지 않습니다 (gh의 라이프사이클 침범 방지).
pub async fn get_token() -> Option<String> {
    // gh --version으로 존재 여부 확인
    let version_check = tokio::process::Command::new("gh")
        .arg("--version")
        .output()
        .await;

    match version_check {
        Ok(out) if out.status.success() => {}
        _ => {
            tracing::debug!("gh CLI가 없거나 실행 불가");
            return None;
        }
    }

    // gh auth token으로 토큰 가져오기
    let token_out = tokio::process::Command::new("gh")
        .args(["auth", "token"])
        .output()
        .await;

    match token_out {
        Ok(out) if out.status.success() => {
            let token = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if token.is_empty() {
                tracing::debug!("gh auth token이 빈 문자열 반환");
                None
            } else {
                tracing::debug!("gh CLI에서 토큰 획득 성공");
                Some(token)
            }
        }
        _ => {
            tracing::debug!("gh auth token 실행 실패");
            None
        }
    }
}
