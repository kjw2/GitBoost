fn main() {
    // GITBOOST_GITHUB_CLIENT_ID를 컴파일타임 환경변수로 주입
    // 환경변수가 설정되지 않으면 기본 Client ID를 사용
    const DEFAULT_CLIENT_ID: &str = "Ov23liys1cnsX7VYUfIQ";
    // env var가 미설정이거나 빈 문자열(GHA에서 secret 미설정 시)인 경우 기본값 사용
    let client_id = std::env::var("GITBOOST_GITHUB_CLIENT_ID")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| DEFAULT_CLIENT_ID.to_string());
    println!("cargo:rustc-env=GITBOOST_GITHUB_CLIENT_ID={}", client_id);
    println!("cargo:rerun-if-env-changed=GITBOOST_GITHUB_CLIENT_ID");
}
