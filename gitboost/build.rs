fn main() {
    // GITBOOST_GITHUB_CLIENT_ID를 컴파일타임 환경변수로 주입
    // 환경변수가 설정되지 않으면 기본 Client ID를 사용
    const DEFAULT_CLIENT_ID: &str = "Ov23liys1cnsX7VYUfIQ";
    let client_id =
        std::env::var("GITBOOST_GITHUB_CLIENT_ID").unwrap_or_else(|_| DEFAULT_CLIENT_ID.to_string());
    println!("cargo:rustc-env=GITBOOST_GITHUB_CLIENT_ID={}", client_id);
    println!("cargo:rerun-if-env-changed=GITBOOST_GITHUB_CLIENT_ID");
}
