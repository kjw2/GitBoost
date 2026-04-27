use gitboost::cli::{Cli, Command};
use gitboost::ui;

use clap::Parser;

fn main() {
    let exit_code = real_main();
    std::process::exit(exit_code);
}

fn real_main() -> i32 {
    let cli = Cli::parse();
    init_tracing(cli.verbose);

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("failed to build tokio runtime");

    let result = runtime.block_on(async move {
        match cli.command {
            Command::Create(args) => gitboost::orchestrator::run_create(args).await,
            Command::Login => commands::login().await,
            Command::Logout => commands::logout().await,
            Command::Whoami => commands::whoami().await,
        }
    });

    match result {
        Ok(()) => 0,
        Err(e) => {
            ui::error_block(&e);
            e.exit_code()
        }
    }
}

/// tracing 초기화: verbose 여부에 따라 로그 레벨 설정
fn init_tracing(verbose: bool) {
    use tracing_subscriber::{fmt, EnvFilter};

    let filter = if verbose {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("gitboost=debug"))
    } else {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("gitboost=info"))
    };

    fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .without_time()
        .init();
}

mod commands {
    use gitboost::auth;
    use gitboost::error::Result;
    use gitboost::github::GithubClient;
    use gitboost::ui;
    use gitboost::GitBoostError;

    /// Device Flow를 강제 실행하여 명시적 로그인 수행
    pub async fn login() -> Result<()> {
        let client = reqwest::Client::builder()
            .user_agent(gitboost::config::USER_AGENT)
            .build()
            .map_err(|e| GitBoostError::Network(e.to_string()))?;

        ui::step("GitHub Device Flow 인증을 시작합니다...");
        let token = auth::device_flow::run_device_flow(&client).await?;
        auth::keyring_storage::save(&token)?;
        ui::info("✓ 인증 성공! 토큰이 keyring에 저장되었습니다.");
        Ok(())
    }

    /// keyring에서 GitBoost 토큰 삭제
    pub async fn logout() -> Result<()> {
        auth::keyring_storage::delete()?;
        ui::info("✓ GitBoost 토큰이 keyring에서 삭제되었습니다.");
        Ok(())
    }

    /// 현재 인증된 GitHub 사용자 정보 출력
    pub async fn whoami() -> Result<()> {
        let client = reqwest::Client::builder()
            .user_agent(gitboost::config::USER_AGENT)
            .build()
            .map_err(|e| GitBoostError::Network(e.to_string()))?;

        let token_source = auth::resolve_token(&client).await?;
        let gh_client = GithubClient::new(client, token_source.token);
        let user = gh_client.get_user().await?;

        let origin_label = match token_source.origin {
            auth::Origin::GhCli => "gh CLI",
            auth::Origin::Keyring => "keyring",
            auth::Origin::DeviceFlow => "device flow",
        };

        ui::info(&format!("GitHub 사용자: {} ({})", user.login, origin_label));
        if let Some(name) = &user.name {
            ui::info(&format!("  이름: {}", name));
        }
        if let Some(email) = &user.email {
            ui::info(&format!("  이메일: {}", email));
        }
        Ok(())
    }
}
