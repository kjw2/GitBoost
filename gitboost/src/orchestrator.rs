use crate::auth;
use crate::cli::CreateArgs;
use crate::error::{GitBoostError, Result};
use crate::generator;
use crate::git_ops;
use crate::github::{CreateRepoRequest, GithubClient};
use crate::prereq;
use crate::ui;
use crate::ui::prompt;

/// `gitboost create` 명령의 전체 흐름을 오케스트레이션합니다.
pub async fn run_create(args: CreateArgs) -> Result<()> {
    // 1. 사전 요구사항 검사
    ui::step("사전 요구사항 확인...");
    prereq::check()?;
    ui::success("사전 요구사항 확인");

    // 2. 저장소 이름 검증
    git_ops::validate_repo_name(&args.name)?;

    // 3. 라이선스 ID 검증
    generator::license::validate(&args.license)?;

    // 4. 디렉토리 충돌 검사 (인증 전에 수행)
    let cwd = std::env::current_dir()
        .map_err(|e| GitBoostError::Fs(format!("현재 디렉토리를 알 수 없습니다: {}", e)))?;
    let project_dir = cwd.join(&args.name);

    ui::step(&format!("디렉토리 검사: ./{}", args.name));
    if project_dir.exists() {
        // 비어있는지 확인
        let entries = std::fs::read_dir(&project_dir)
            .map_err(|e| GitBoostError::Fs(format!("디렉토리 읽기 실패: {}", e)))?
            .count();
        if entries > 0 {
            return Err(GitBoostError::Fs(format!(
                "'{}' 디렉토리가 이미 존재하며 비어있지 않습니다.\n  \
                 다른 이름을 사용하거나 해당 디렉토리를 비워주세요.",
                args.name
            )));
        }
    }
    ui::success(&format!("디렉토리 검사: ./{}", args.name));

    // 5. 인증
    ui::step("GitHub 인증 확인...");
    let http_client = reqwest::Client::builder()
        .user_agent(crate::config::USER_AGENT)
        .build()
        .map_err(|e| GitBoostError::Network(e.to_string()))?;

    let token_source = auth::resolve_token(&http_client).await?;
    let origin_label = match token_source.origin {
        auth::Origin::GhCli => "gh CLI 위임",
        auth::Origin::Keyring => "keyring",
        auth::Origin::DeviceFlow => "device flow",
    };

    let gh_client = GithubClient::new(http_client, token_source.token.clone());

    // 6. 사용자 정보 조회
    let user = gh_client.get_user().await?;
    ui::success(&format!("GitHub 인증 ({}) - {}", origin_label, user.login));

    // 7. 원격 저장소 이름 충돌 검사
    ui::step("원격 저장소 이름 충돌 검사...");
    if let Some(_existing) = gh_client.get_repo(&user.login, &args.name).await? {
        return Err(GitBoostError::GitHub {
            status: 422,
            message: format!(
                "GitHub에 '{}' 저장소가 이미 존재합니다.\n  \
                 다른 이름을 사용하세요.",
                args.name
            ),
        });
    }
    ui::success("원격 저장소 이름 충돌 검사");

    // 8. 디렉토리 생성
    if !project_dir.exists() {
        std::fs::create_dir_all(&project_dir)
            .map_err(|e| GitBoostError::Fs(format!("디렉토리 생성 실패: {}", e)))?;
    }

    // 저작자 결정: --author > git config user.name > GitHub user name/login
    let author = args.author.clone().unwrap_or_else(|| {
        git_ops::get_git_user_name(&project_dir)
            .or_else(|| user.name.clone())
            .unwrap_or_else(|| user.login.clone())
    });

    // 커밋용 이메일 결정
    let commit_email = git_ops::get_git_user_email(&project_dir)
        .or_else(|| user.email.clone())
        .unwrap_or_else(|| format!("{}@users.noreply.github.com", user.login));

    // 9. 파일 생성
    ui::step("파일 생성 (README, LICENSE, .gitignore)...");
    generator::write_files(generator::WriteFilesArgs {
        project_name: &args.name,
        license_id: &args.license,
        author: &author,
        description: args.description.as_deref(),
        template: args.template.as_deref(),
        dir: &project_dir,
        github_client: Some(&gh_client),
    })
    .await?;
    ui::success("파일 생성 (README, LICENSE, .gitignore)");

    // 10. git init
    ui::step("Git 초기화...");
    git_ops::init_repo(&project_dir)?;
    ui::success("Git 초기화");

    // 11. git add + initial commit
    ui::step("초기 커밋 생성...");
    git_ops::stage_all(&project_dir)?;
    git_ops::initial_commit(
        &project_dir,
        "chore: initial commit by GitBoost",
        &author,
        &commit_email,
    )?;
    ui::success("초기 커밋 생성");

    // 12. GitHub 저장소 생성
    let visibility = if args.public { "public" } else { "private" };
    ui::step(&format!("GitHub 저장소 생성 ({})...", visibility));
    let repo = gh_client
        .create_repo(&CreateRepoRequest {
            name: &args.name,
            private: !args.public,
            description: args.description.as_deref(),
            auto_init: false,
        })
        .await?;
    ui::success(&format!("GitHub 저장소 생성 ({})", visibility));

    // 13. remote 등록
    ui::step("origin 등록...");
    git_ops::add_remote(&project_dir, "origin", &repo.clone_url)?;
    ui::success("origin 등록");

    // 14. push
    if !args.no_push {
        ui::step("첫 push...");
        let push_result = git_ops::push(&project_dir, &token_source.token, "origin", "main");

        if let Err(push_err) = push_result {
            ui::fail("첫 push 실패");

            // 롤백 여부 사용자에게 묻기
            let should_delete = prompt::confirm(
                &format!(
                    "push에 실패했습니다. GitHub 저장소 '{}'을 삭제하시겠습니까?",
                    repo.full_name
                ),
                false,
                args.yes,
            )?;

            if should_delete {
                ui::step("원격 저장소 삭제 (롤백)...");
                match gh_client.delete_repo(&user.login, &args.name).await {
                    Ok(()) => ui::success("원격 저장소 삭제 완료"),
                    Err(e) => ui::warn(&format!("원격 저장소 삭제 실패: {}", e)),
                }
            } else {
                ui::info(&format!("원격 저장소가 남아있습니다: {}", repo.html_url));
                ui::info(&format!(
                    "수동으로 push하려면: git -C {} push -u origin main",
                    project_dir.display()
                ));
            }

            return Err(push_err);
        }

        ui::success("첫 push");
    }

    // 15. 완료 메시지
    println!();
    println!("🎉 완료! {}", repo.html_url);
    println!("   다음으로:  cd {}", args.name);
    if args.no_push {
        println!("   push하려면: cd {} && git push -u origin main", args.name);
    }

    Ok(())
}
