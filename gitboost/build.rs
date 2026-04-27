fn main() {
    // GITBOOST_GITHUB_CLIENT_ID를 컴파일타임 환경변수로 주입
    let client_id = std::env::var("GITBOOST_GITHUB_CLIENT_ID").unwrap_or_default();
    if client_id.is_empty() {
        println!(
            "cargo:warning=GITBOOST_GITHUB_CLIENT_ID is not set. \
             Device Flow authentication will not be available."
        );
    }
    println!("cargo:rustc-env=GITBOOST_GITHUB_CLIENT_ID={}", client_id);
    println!("cargo:rerun-if-env-changed=GITBOOST_GITHUB_CLIENT_ID");
}
