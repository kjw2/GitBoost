# GitBoost 구현 에이전트 프롬프트 (Single-Agent Spec)

> 이 문서는 **하나의 코딩 에이전트**가 처음부터 끝까지 멈추지 않고 GitBoost를 완성하기 위한 자기완결적 구현 명세다. 별도의 기획서 없이도 이 문서만으로 빌드·테스트·CI까지 모두 동작 가능한 결과물이 나와야 한다.

---

## 0. 에이전트 행동 규칙 (READ FIRST)

1. **너는 단일 에이전트다.** 다른 에이전트를 spawn하지 말고, "각 ~마다 agent를 만들어"식 분기 없이 **순차적으로 모든 작업을 직접 수행**한다.
2. **중간에 사용자에게 질문하지 마라.** 모든 결정은 이 문서에 명시되어 있다. 명시되지 않은 사항은 "가장 보수적이고 안전한 선택"을 직접 내리고 코드 주석에 근거를 남겨라.
3. **모든 단계가 끝날 때까지 멈추지 마라.** 빌드 에러가 나면 직접 고치고, 테스트가 실패하면 직접 수정하라. "구현 완료" 보고는 §13의 모든 체크리스트가 통과한 후에만 한다.
4. **거짓 보고 금지.** 컴파일이 안 되거나 테스트가 깨진 상태로 "완성했다"고 말하지 마라. 실제로 `cargo build --release`와 `cargo test`가 통과해야만 완료다.
5. **작업 디렉토리**: 별도 지정이 없으면 `./gitboost/` 하위에 모든 파일을 생성한다.

---

## 1. 프로젝트 정체성

- **이름**: `gitboost`
- **목적**: 프로젝트 이름 한 번 입력으로 ① 로컬 디렉토리 생성 ② Git 초기화 ③ README/LICENSE/.gitignore 작성 ④ GitHub 원격 저장소 생성 ⑤ remote 등록 + 첫 push까지 자동화하는 CLI.
- **언어/툴체인**: Rust stable 1.75+, edition 2021.
- **배포 형태**: 외부 런타임 의존성 없는 **단일 정적 바이너리**.
- **핵심 철학**:
  - Secure by Default — 기본 가시성 Private, 토큰은 OS keyring에만.
  - Zero-Config — `gh` CLI 인증이 있으면 그것을 그대로 쓴다.
  - Idempotent & Safe — 파괴적 동작 전에 확인 절차. 데이터 손실 금지.
  - Fail Loud — 에러는 사용자가 다음에 무엇을 해야 하는지 알 수 있도록 출력.

---

## 2. Cargo.toml 정확한 사양

```toml
[package]
name = "gitboost"
version = "0.1.0"
edition = "2021"
rust-version = "1.75"
description = "Zero-config CLI to create a local + GitHub repo in one command"
license = "MIT"
repository = "https://github.com/your-org/gitboost"
keywords = ["git", "github", "cli", "automation", "scaffold"]
categories = ["command-line-utilities", "development-tools"]

[[bin]]
name = "gitboost"
path = "src/main.rs"

[lib]
name = "gitboost"
path = "src/lib.rs"

[dependencies]
clap = { version = "4.5", features = ["derive", "env"] }
tokio = { version = "1.40", features = ["rt-multi-thread", "macros", "process", "time", "signal"] }
reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "1"
anyhow = "1"
keyring = "3"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt"] }
owo-colors = { version = "4", features = ["supports-colors"] }
indicatif = "0.17"
dialoguer = { version = "0.11", default-features = false }
url = "2"
chrono = { version = "0.4", default-features = false, features = ["clock"] }
webbrowser = "1"

[dev-dependencies]
assert_cmd = "2"
predicates = "3"
tempfile = "3"
wiremock = "0.6"

[build-dependencies]
# build.rs는 외부 크레이트 없이 std만 사용

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true
panic = "abort"
```

> **주의**: `reqwest`는 반드시 `default-features = false` + `rustls-tls`로 설정하여 OpenSSL 의존성을 제거한다. 이게 cross-compile 성공의 핵심이다.

---

## 3. 디렉토리 구조 (정확히 이대로 만든다)

```
gitboost/
├── Cargo.toml
├── README.md
├── LICENSE
├── CHANGELOG.md
├── .gitignore
├── build.rs
├── .github/
│   └── workflows/
│       ├── ci.yml
│       └── release.yml
├── tests/
│   └── cli_smoke.rs
└── src/
    ├── main.rs
    ├── lib.rs
    ├── cli.rs
    ├── error.rs
    ├── config.rs
    ├── prereq.rs
    ├── auth/
    │   ├── mod.rs
    │   ├── gh_cli.rs
    │   ├── keyring_storage.rs
    │   └── device_flow.rs
    ├── github/
    │   ├── mod.rs
    │   ├── models.rs
    │   └── repos.rs
    ├── generator/
    │   ├── mod.rs
    │   ├── readme.rs
    │   ├── license.rs
    │   ├── gitignore.rs
    │   └── templates/
    │       ├── mit.txt
    │       ├── apache-2.0.txt
    │       ├── gpl-3.0.txt
    │       ├── bsd-3-clause.txt
    │       ├── mpl-2.0.txt
    │       └── unlicense.txt
    ├── git_ops/
    │   ├── mod.rs
    │   ├── runner.rs
    │   └── credential.rs
    ├── ui/
    │   ├── mod.rs
    │   └── prompt.rs
    └── orchestrator.rs
```

---

## 4. 모듈별 구현 명세

### 4.1. `build.rs`
- 환경변수 `GITBOOST_GITHUB_CLIENT_ID`를 읽는다.
- 비어있거나 없으면 빈 문자열로 처리하되 `cargo:warning`을 출력한다 (CI는 통과하되 production 빌드는 명시적 경고).
- `cargo:rustc-env=GITBOOST_GITHUB_CLIENT_ID=<값>` 형태로 컴파일타임 변수로 주입.
- `cargo:rerun-if-env-changed=GITBOOST_GITHUB_CLIENT_ID` 선언.

### 4.2. `src/config.rs`
다음 상수를 정의한다:
```rust
pub const APP_NAME: &str = "gitboost";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const USER_AGENT: &str = concat!("gitboost/", env!("CARGO_PKG_VERSION"));
pub const KEYRING_SERVICE: &str = "gitboost";
pub const KEYRING_USER: &str = "github_token";
pub const GITHUB_CLIENT_ID: &str = env!("GITBOOST_GITHUB_CLIENT_ID");
pub const GITHUB_API_BASE_DEFAULT: &str = "https://api.github.com";
pub const GITHUB_OAUTH_BASE: &str = "https://github.com";

pub fn github_api_base() -> String {
    std::env::var("GITHUB_API_BASE").unwrap_or_else(|_| GITHUB_API_BASE_DEFAULT.to_string())
}
```
- `env!("GITBOOST_GITHUB_CLIENT_ID")`가 빌드 시 정의되지 않은 경우 컴파일이 깨진다. 이를 막기 위해 `build.rs`에서 항상 빈 문자열이라도 주입한다 (위 §4.1).
- 런타임에 `GITHUB_CLIENT_ID`가 빈 문자열이면 Device Flow 호출 시 명확한 에러를 던진다.

### 4.3. `src/error.rs`
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GitBoostError {
    #[error("사전 요구사항 미충족: {0}")]
    Prerequisite(String),
    #[error("인증 실패: {0}")]
    Auth(String),
    #[error("GitHub API 에러 ({status}): {message}")]
    GitHub { status: u16, message: String },
    #[error("로컬 파일시스템 에러: {0}")]
    Fs(String),
    #[error("Git 명령 실패: {cmd}\n  stderr: {stderr}")]
    Git { cmd: String, stderr: String },
    #[error("네트워크 에러: {0}")]
    Network(String),
    #[error("사용자가 작업을 취소했습니다")]
    UserAborted,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl GitBoostError {
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Prerequisite(_) => 10,
            Self::Auth(_) => 11,
            Self::GitHub { .. } => 12,
            Self::Fs(_) => 13,
            Self::Git { .. } => 14,
            Self::Network(_) => 15,
            Self::UserAborted => 1,
            Self::Other(_) => 1,
        }
    }
    pub fn next_action_hint(&self) -> Option<&'static str> {
        match self {
            Self::Auth(_) => Some("`gitboost login`을 실행해 다시 인증하세요."),
            Self::Prerequisite(_) => Some("필수 도구를 설치한 뒤 다시 시도하세요."),
            Self::GitHub { status: 401 | 403, .. } => Some("`gitboost logout` 후 `gitboost login`으로 토큰을 갱신하세요."),
            _ => None,
        }
    }
}

pub type Result<T> = std::result::Result<T, GitBoostError>;
```

### 4.4. `src/cli.rs`
clap derive로 다음 구조를 정의:
```rust
#[derive(Parser, Debug)]
#[command(name = "gitboost", version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
    #[arg(short, long, global = true)]
    pub verbose: bool,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// 새 프로젝트와 GitHub 저장소를 한 번에 생성합니다
    Create(CreateArgs),
    /// GitHub Device Flow로 명시적 로그인을 수행합니다
    Login,
    /// keyring에 저장된 GitBoost 토큰을 삭제합니다
    Logout,
    /// 현재 인증된 GitHub 사용자 정보를 출력합니다
    Whoami,
}

#[derive(Args, Debug)]
pub struct CreateArgs {
    /// 프로젝트(=저장소) 이름. 영문/숫자/하이픈/언더스코어만 허용
    pub name: String,
    /// Public 저장소로 생성 (기본은 Private)
    #[arg(long)]
    pub public: bool,
    /// SPDX 라이선스 ID (mit/apache-2.0/gpl-3.0/bsd-3-clause/mpl-2.0/unlicense/none)
    #[arg(short, long, default_value = "mit")]
    pub license: String,
    /// .gitignore 템플릿 (rust/node/python/go/java 등)
    #[arg(short, long)]
    pub template: Option<String>,
    /// 저장소 설명
    #[arg(short, long)]
    pub description: Option<String>,
    /// 라이선스 저작자 이름 (기본: git config user.name → GitHub user.name)
    #[arg(long)]
    pub author: Option<String>,
    /// 원격 저장소는 만들되 첫 push는 생략
    #[arg(long)]
    pub no_push: bool,
    /// 모든 확인 프롬프트에 자동으로 동의
    #[arg(short, long)]
    pub yes: bool,
}
```
- 저장소 이름 검증 정규식: `^[A-Za-z0-9._-]{1,100}$`. 위반 시 즉시 에러.

### 4.5. `src/prereq.rs`
- `git --version`을 실행하고, 출력에서 버전을 파싱한다 (예: `git version 2.43.0`).
- 2.28 미만이면 `Prerequisite` 에러.
- `gh --version`은 선택 사항이므로 결과만 반환 (`bool`).

### 4.6. `src/auth/`

#### 4.6.1. `auth/mod.rs`
```rust
pub async fn resolve_token(client: &reqwest::Client) -> Result<TokenSource>
```
- Level 1 → Level 2 → Level 3 순서.
- `TokenSource { token: String, origin: Origin }`로 반환. `Origin`은 enum (`GhCli`, `Keyring`, `DeviceFlow`).
- 각 단계에서 실제 GitHub API `/user` 호출로 검증 (200이면 성공).
- 401/403이면 다음 단계로 폴백 (단 keyring에 저장된 토큰이 무효한 경우 keyring에서 삭제).

#### 4.6.2. `auth/gh_cli.rs`
- `gh --version` 체크 → 없으면 `None` 반환.
- `gh auth token` 실행 → stdout trim → 비어있지 않으면 토큰 반환.
- gh CLI 토큰은 keyring에 **저장하지 않는다**.

#### 4.6.3. `auth/keyring_storage.rs`
```rust
pub fn save(token: &str) -> Result<()>
pub fn load() -> Result<Option<String>>
pub fn delete() -> Result<()>
```
- `keyring::Entry::new("gitboost", "github_token")` 사용.
- `NoEntry` 에러는 `Ok(None)` / `Ok(())`로 정상 처리.

#### 4.6.4. `auth/device_flow.rs`
- `POST https://github.com/login/device/code` body:
  - `client_id={CLIENT_ID}&scope=repo`
  - 헤더: `Accept: application/json`
- 응답을 받으면 사용자에게:
  ```
  ▸ 브라우저에서 다음 URL을 열고 코드를 입력하세요:
      URL : https://github.com/login/device
      CODE: ABCD-1234
  ```
- `webbrowser::open(verification_uri)`를 시도하되 실패해도 진행.
- `interval`초 간격(기본 5)으로 `POST https://github.com/login/oauth/access_token`:
  - body: `client_id={CLIENT_ID}&device_code={code}&grant_type=urn:ietf:params:oauth:grant-type:device_code`
- 응답 분기:
  - `{"access_token": ...}` → 성공, keyring에 저장 후 반환.
  - `{"error": "authorization_pending"}` → 계속.
  - `{"error": "slow_down"}` → interval += 5.
  - `{"error": "expired_token"}` → `Auth` 에러 종료.
  - `{"error": "access_denied"}` → `Auth` 에러 종료.
  - 그 외 에러 → `Auth` 에러 종료.
- 최대 폴링 시간: `expires_in`초 (응답에서 받음). 초과 시 timeout 에러.
- `tokio::select!`로 `tokio::signal::ctrl_c()`도 함께 대기 → Ctrl+C 시 `UserAborted`.

### 4.7. `src/github/`

#### 4.7.1. `github/mod.rs`
- `GithubClient`: `reqwest::Client`와 토큰을 보유.
- 모든 요청에 자동으로 헤더 부착:
  - `Authorization: Bearer <token>`
  - `Accept: application/vnd.github+json`
  - `User-Agent: gitboost/<version>`
  - `X-GitHub-Api-Version: 2022-11-28`
- 응답 처리: 4xx/5xx면 `GitHubError` 변환 (응답 body의 `message` 필드 사용, 파싱 실패 시 status text).

#### 4.7.2. `github/models.rs`
```rust
#[derive(Deserialize)]
pub struct User {
    pub login: String,
    pub name: Option<String>,
    pub email: Option<String>,
}

#[derive(Deserialize)]
pub struct Repo {
    pub name: String,
    pub full_name: String,
    pub html_url: String,
    pub clone_url: String,
    pub ssh_url: String,
    pub private: bool,
}

#[derive(Serialize)]
pub struct CreateRepoRequest<'a> {
    pub name: &'a str,
    pub private: bool,
    pub description: Option<&'a str>,
    pub auto_init: bool, // 항상 false
}

#[derive(Deserialize)]
pub struct GitignoreTemplate {
    pub name: String,
    pub source: String,
}

#[derive(Deserialize)]
pub struct ApiErrorBody {
    pub message: String,
}
```

#### 4.7.3. `github/repos.rs`
```rust
pub async fn get_user(&self) -> Result<User>
pub async fn get_repo(&self, owner: &str, repo: &str) -> Result<Option<Repo>>  // 404면 None
pub async fn create_repo(&self, req: &CreateRepoRequest<'_>) -> Result<Repo>
pub async fn delete_repo(&self, owner: &str, repo: &str) -> Result<()>
pub async fn fetch_gitignore_template(&self, name: &str) -> Result<GitignoreTemplate>
```

### 4.8. `src/generator/`

#### 4.8.1. `generator/license.rs`
- 6개 라이선스 본문은 `include_str!("templates/<id>.txt")`로 임베드.
- MIT/BSD-3-Clause 본문에는 `{{YEAR}}`, `{{AUTHOR}}` 플레이스홀더를 두고 런타임에 치환.
- 다른 라이선스는 치환 없이 그대로 사용.
- 입력 SPDX ID 정규화: 소문자, 하이픈 통일. 매핑 테이블:
  ```
  mit, apache-2.0, gpl-3.0, bsd-3-clause, mpl-2.0, unlicense, none
  ```
- 미지원 ID면 `Prerequisite` 에러로 명확한 메시지.

#### 4.8.2. `generator/gitignore.rs`
- 사용자 입력 → GitHub API의 정확한 이름으로 매핑하는 테이블:
  ```
  rust → Rust, node → Node, python → Python, go → Go, java → Java,
  cpp/c++ → C++, c → C, csharp/cs → CSharp, ruby → Ruby, php → PHP,
  swift → Swift, kotlin → Kotlin, scala → Scala, elixir → Elixir,
  haskell → Haskell, dart → Dart, lua → Lua, perl → Perl,
  unity → Unity, unreal → UnrealEngine, godot → Godot, android → Android
  ```
- 매핑에 없으면 입력값 그대로 시도 (사용자가 정확한 이름을 안다고 가정).
- 다운로드 실패는 경고로 처리하고 계속 진행 (치명적 아님). 단, 사용자가 명시적으로 `-t`를 줬으므로 경고 메시지는 명확하게.

#### 4.8.3. `generator/readme.rs`
- 내용: `# {project_name}\n` (단순). 추가 옵션 없음.

#### 4.8.4. `generator/mod.rs`
오케스트레이션:
1. README 생성 (이미 있으면 스킵, 경고).
2. LICENSE 생성 (license != "none"인 경우).
3. .gitignore 생성 (template 옵션이 있는 경우).
- 모든 파일 쓰기는 `std::fs::write`로 atomic하게.
- 파일이 이미 존재하면 덮어쓰지 않는다.

#### 4.8.5. 라이선스 템플릿 파일 내용
**`src/generator/templates/mit.txt`** — 표준 MIT 라이선스 본문에서 연도/이름 부분만 `{{YEAR}}`, `{{AUTHOR}}`로 치환. (OSI 공식 본문 사용.)

**`src/generator/templates/apache-2.0.txt`**, **`gpl-3.0.txt`**, **`mpl-2.0.txt`** — 각각의 표준 전문(full text)을 그대로. 치환 없음.

**`src/generator/templates/bsd-3-clause.txt`** — 표준 본문에서 연도/이름을 `{{YEAR}}`, `{{AUTHOR}}`로 치환.

**`src/generator/templates/unlicense.txt`** — Unlicense 표준 본문 그대로.

> 에이전트 지시: 각 라이선스의 정확한 표준 본문은 https://opensource.org/licenses/ 와 https://www.gnu.org/licenses/gpl-3.0.txt 의 공식 텍스트를 사용한다. 본문이 매우 긴 라이선스(GPL-3.0, Apache-2.0)의 경우 줄바꿈/공백을 원본 그대로 보존한다. 본문에 `{`나 `}` 문자가 있어도 Rust의 `include_str!`는 영향받지 않으므로 그대로 둔다.

### 4.9. `src/git_ops/`

#### 4.9.1. `git_ops/runner.rs`
```rust
pub struct GitRunner { cwd: PathBuf }
impl GitRunner {
    pub fn new(cwd: impl Into<PathBuf>) -> Self;
    pub fn run(&self, args: &[&str]) -> Result<String>;
    pub fn run_with_env(&self, args: &[&str], env: &[(&str, &str)]) -> Result<String>;
}
```
- 내부적으로 `Command::new("git").current_dir(&self.cwd).args(args)` 실행.
- stderr를 캡처해 `Git { cmd, stderr }` 에러로 변환.
- stdin 비활성화: `.stdin(Stdio::null())`.

#### 4.9.2. `git_ops/credential.rs`
Push 시 토큰 주입을 위한 임시 credential helper.
- **Unix**: `git -c credential.helper= -c credential.helper='!f() { echo "username=x-access-token"; echo "password=$GITBOOST_TOKEN"; }; f' push ...` 형태로 실행. `GITBOOST_TOKEN` 환경변수를 자식 프로세스에만 전달.
- **Windows (`cfg(windows)`)**: 임시 디렉토리에 다음 내용의 `gitboost-cred-<pid>.bat` 파일을 생성:
  ```
  @echo off
  echo username=x-access-token
  echo password=%GITBOOST_TOKEN%
  ```
  그리고 `git -c credential.helper= -c credential.helper="<absolute_path_to_bat>"`로 실행. **반드시 try/finally(또는 Drop)로 파일을 삭제**.
- 중요: 토큰을 절대 로그/stdout/stderr/명령행에 노출하지 말 것. clap 인자나 tracing 출력에서도 마스킹.

#### 4.9.3. `git_ops/mod.rs`
순서대로 다음 함수들:
```rust
pub fn init_repo(cwd: &Path) -> Result<()>;
pub fn stage_all(cwd: &Path) -> Result<()>;
pub fn initial_commit(cwd: &Path, message: &str) -> Result<()>;
pub fn add_remote(cwd: &Path, name: &str, url: &str) -> Result<()>;
pub fn push(cwd: &Path, token: &str, remote: &str, branch: &str) -> Result<()>;
```
- `init_repo`: 먼저 `git init --initial-branch=main` 시도, 실패 시 `git init` 후 `git symbolic-ref HEAD refs/heads/main` 폴백.
- `initial_commit`: `git -c commit.gpgsign=false -c user.name=... -c user.email=... commit -m ...`. user.name/email은 호출자가 미리 결정해 인자로 전달.
- `push`: `git_ops/credential.rs`의 헬퍼를 사용. `-u origin main`.

### 4.10. `src/ui/`

#### 4.10.1. `ui/mod.rs`
- `step(label)`: cyan `▸ <label>` 출력.
- `success(label)`: 같은 줄을 `✓`로 마무리하는 spinner 종료.
- `fail(label)`: red `✗`로 종료.
- `info(msg)`, `warn(msg)`, `error_block(err)` 등.
- TTY/NO_COLOR 검사 후 spinner/컬러 비활성화.

#### 4.10.2. `ui/prompt.rs`
- `confirm(question, default, yes_flag)`:
  - `yes_flag`가 true면 무조건 default 반환 (대화형 우회).
  - stdin이 TTY가 아니면 default 반환.
  - 그 외에는 `dialoguer::Confirm` 사용.

### 4.11. `src/orchestrator.rs`
`pub async fn run_create(args: CreateArgs) -> Result<()>` — 전체 흐름 오케스트레이션. **이 함수가 GitBoost의 핵심**.

순서:
```
1. 사전 요구사항 (prereq::check)
2. 저장소 이름 검증 (정규식)
3. 라이선스 ID 검증 (generator::license::validate)
4. CWD 기준 디렉토리 충돌 검사
   - <cwd>/<name>이 존재하고 비어있지 않으면 에러 종료 (Fs)
5. 인증 (auth::resolve_token)
6. 사용자 정보 조회 (github::get_user) — owner와 라이선스 author 결정에 사용
7. 원격 저장소 이름 충돌 검사 (github::get_repo(owner, name))
   - Some이면 에러 종료
8. 디렉토리 생성 (없으면 mkdir)
9. 파일 생성 (generator::write_files)
   - README, LICENSE (license != none), .gitignore (template 있을 때)
10. git init (git_ops::init_repo)
11. git add . + initial commit (git_ops::stage_all + initial_commit)
12. github::create_repo
13. git_ops::add_remote("origin", clone_url)
14. (--no-push이 아니면) git_ops::push
    - 실패 시: 사용자에게 원격 저장소 삭제 여부 confirm.
      - yes → github::delete_repo
      - no  → 메시지만 출력
    - 어느 쪽이든 push 에러 자체는 그대로 반환 (Git 에러)
15. 성공 메시지 출력 (Repo URL + 다음 단계 안내)
```

각 단계는 `ui::step("...")` → 수행 → `ui::success("...")`로 시각화.

---

## 5. `src/main.rs` 정확한 동작

```rust
// pseudocode 수준이지만 이대로 구현할 것
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
            Command::Create(args) => orchestrator::run_create(args).await,
            Command::Login        => commands::login().await,
            Command::Logout       => commands::logout(),
            Command::Whoami       => commands::whoami().await,
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
```

`init_tracing`:
- `verbose`면 `RUST_LOG`가 없을 때 `gitboost=debug` 적용, 있으면 그대로.
- `verbose`가 아니면 `gitboost=info`.
- `tracing_subscriber::fmt()`로 stderr에 출력. 토큰이 포함된 필드는 절대 로그하지 않도록 코드에서 마스킹.

`commands::login`: Device Flow 강제 실행 (gh CLI/keyring 우회) 후 keyring에 저장.
`commands::logout`: keyring에서 토큰 삭제. 없어도 성공.
`commands::whoami`: `resolve_token` → `get_user` → `login (origin: gh-cli/keyring/device-flow)` 출력.

---

## 6. 테스트 (`tests/cli_smoke.rs`)

다음 테스트는 **반드시 통과**해야 한다 (네트워크 없이):

```rust
#[test]
fn version_flag_prints_version();          // `gitboost --version` → exit 0, "gitboost" 포함

#[test]
fn help_flag_prints_help();                // `gitboost --help` → exit 0

#[test]
fn create_without_name_fails();            // `gitboost create` → exit 2

#[test]
fn create_invalid_name_fails();            // `gitboost create "in valid"` → 0이 아님, 에러 메시지에 "이름" 포함

#[test]
fn create_existing_nonempty_dir_fails();   // tempdir에 파일 둔 후 `gitboost create <dir>` → 종료코드 13
                                            // 단, 인증/네트워크 단계 전에 실패해야 한다 → 인증 단계 진입 전 디렉토리 검사를 한다는 §4.11의 순서 보장
```

> **중요**: `create_existing_nonempty_dir_fails` 테스트는 인증 모듈이 호출되기 전에 디렉토리 검사가 실패하도록 §4.11의 순서를 변경할 것 — 순서를 다음과 같이 조정한다: **1→2→3→4(디렉토리 검사)→5(인증)...**. 이미 위 §4.11에 그렇게 적혀 있다.

단위 테스트는 각 모듈 파일 내부 `#[cfg(test)] mod tests`로 추가:
- `generator/license.rs`: `{{YEAR}}`, `{{AUTHOR}}` 치환 테스트.
- `generator/gitignore.rs`: 입력 정규화 테스트 (`rust` → `Rust` 등).
- `cli.rs`: 저장소 이름 검증 정규식.

---

## 7. CI: `.github/workflows/ci.yml`

```yaml
name: CI
on:
  push:
    branches: [main]
  pull_request:

env:
  CARGO_TERM_COLOR: always
  GITBOOST_GITHUB_CLIENT_ID: ""

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt
      - uses: Swatinem/rust-cache@v2
      - run: cargo fmt --all -- --check
      - run: cargo clippy --all-targets --all-features -- -D warnings
      - run: cargo test --all
```

---

## 8. CD: `.github/workflows/release.yml`

```yaml
name: Release
on:
  push:
    tags: ['v*']

permissions:
  contents: write

env:
  CARGO_TERM_COLOR: always
  GITBOOST_GITHUB_CLIENT_ID: ${{ secrets.GITBOOST_GITHUB_CLIENT_ID }}

jobs:
  build:
    name: Build ${{ matrix.target }}
    strategy:
      fail-fast: false
      matrix:
        include:
          - { os: ubuntu-latest,  target: x86_64-unknown-linux-gnu,    cross: false, ext: ""    }
          - { os: ubuntu-latest,  target: aarch64-unknown-linux-gnu,   cross: true,  ext: ""    }
          - { os: macos-latest,   target: x86_64-apple-darwin,         cross: false, ext: ""    }
          - { os: macos-latest,   target: aarch64-apple-darwin,        cross: false, ext: ""    }
          - { os: windows-latest, target: x86_64-pc-windows-msvc,      cross: false, ext: ".exe"}
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      - uses: Swatinem/rust-cache@v2

      - name: Install cross (if needed)
        if: matrix.cross == true
        run: cargo install cross --locked

      - name: Build (cross)
        if: matrix.cross == true
        run: cross build --release --target ${{ matrix.target }}

      - name: Build (cargo)
        if: matrix.cross == false
        run: cargo build --release --target ${{ matrix.target }}

      - name: Package
        shell: bash
        run: |
          BIN_NAME="gitboost${{ matrix.ext }}"
          STAGE_DIR="gitboost-${GITHUB_REF_NAME}-${{ matrix.target }}"
          mkdir "$STAGE_DIR"
          cp "target/${{ matrix.target }}/release/${BIN_NAME}" "$STAGE_DIR/"
          cp README.md LICENSE "$STAGE_DIR/" || true
          if [[ "${{ matrix.os }}" == "windows-latest" ]]; then
            7z a "${STAGE_DIR}.zip" "$STAGE_DIR"
            echo "ASSET=${STAGE_DIR}.zip" >> $GITHUB_ENV
          else
            tar -czf "${STAGE_DIR}.tar.gz" "$STAGE_DIR"
            echo "ASSET=${STAGE_DIR}.tar.gz" >> $GITHUB_ENV
          fi

      - name: Checksum
        shell: bash
        run: |
          if [[ "${{ matrix.os }}" == "macos-latest" ]]; then
            shasum -a 256 "$ASSET" > "$ASSET.sha256"
          elif [[ "${{ matrix.os }}" == "windows-latest" ]]; then
            certutil -hashfile "$ASSET" SHA256 | sed -n 2p > "$ASSET.sha256"
          else
            sha256sum "$ASSET" > "$ASSET.sha256"
          fi

      - name: Upload to Release
        uses: softprops/action-gh-release@v2
        with:
          files: |
            ${{ env.ASSET }}
            ${{ env.ASSET }}.sha256
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

---

## 9. README.md / CHANGELOG.md / LICENSE / .gitignore (프로젝트 자체)

- **README.md**: 다음 섹션 포함 — 한 줄 소개, 설치(`cargo install --path .` 및 GitHub Releases), 빠른 시작 (`gitboost create my-app`), 명령어 표, 인증 방식 설명, 라이선스.
- **CHANGELOG.md**: Keep a Changelog 형식, 첫 항목으로 `## [0.1.0] - <오늘 날짜>` + Added 섹션.
- **LICENSE**: MIT, `<현재 연도> GitBoost Contributors`.
- **`.gitignore`**: `/target`, `/Cargo.lock`은 **커밋 포함**(바이너리 프로젝트이므로), `**/*.rs.bk`, `.env`, `.idea/`, `.vscode/`.

---

## 10. 보안 체크리스트 (구현 중 수시 확인)

- [ ] 토큰이 포함된 변수에 `Debug`를 derive하지 말 것 (수동 구현 시 마스킹: `"***"`).
- [ ] tracing 매크로에 토큰을 넘기지 말 것. URL을 로그할 때 쿼리스트링/Authorization 헤더는 제외.
- [ ] Windows credential helper `.bat` 파일은 try/finally(또는 RAII Drop)로 반드시 삭제.
- [ ] Device Flow polling 중 Ctrl+C 처리 → keyring에 부분 토큰 저장 안 함.
- [ ] `gh auth token`으로 받은 토큰은 keyring에 저장하지 않음 (gh의 라이프사이클 침범 방지).
- [ ] reqwest는 rustls-tls만 사용 (OpenSSL 의존성 제거).

---

## 11. 코드 품질 규칙

- `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` 모두 통과해야 완료.
- `unwrap()`/`expect()`는 다음 경우에만 허용:
  - `#[cfg(test)]` 코드 안.
  - `tokio::runtime::Builder::new_multi_thread().build()` 같은 절대 실패하지 않는 초기화.
  - 정규식 컴파일 같은 컴파일타임 보장 가능한 부분 (단 사유를 주석으로).
- 모든 public 함수에는 doc comment (한 줄이라도).
- `pub` 노출 최소화: 모듈 내부에만 쓰는 항목은 `pub(crate)` 또는 비공개.

---

## 12. 구현 순서 (이대로 진행하라)

이 순서대로 작업하면 중간 단계에서도 항상 컴파일 가능한 상태가 유지된다.

1. **Step 1** — 디렉토리/Cargo.toml/build.rs/.gitignore/README.md/LICENSE/CHANGELOG.md 생성.
2. **Step 2** — `src/main.rs`, `src/lib.rs`, `src/config.rs`, `src/error.rs` (최소 구현). `cargo build` 통과 확인.
3. **Step 3** — `src/cli.rs`. `gitboost --version` / `gitboost --help` 동작 확인.
4. **Step 4** — `src/ui/`, `src/prereq.rs`. 사전 요구사항 검사 동작 확인.
5. **Step 5** — `src/generator/templates/*.txt` 6개 라이선스 파일 작성 (정확한 표준 본문).
6. **Step 6** — `src/generator/` 전체 구현 + 단위 테스트.
7. **Step 7** — `src/github/` 전체 구현 + models 단위 테스트.
8. **Step 8** — `src/auth/` 3단계 구현. `gitboost whoami` 동작 확인 (수동).
9. **Step 9** — `src/git_ops/` 구현. credential helper의 Unix/Windows 분기 모두 구현.
10. **Step 10** — `src/orchestrator.rs` 구현. `gitboost create` 종단 흐름 완성.
11. **Step 11** — `tests/cli_smoke.rs` 통합 테스트 작성. 모두 통과 확인.
12. **Step 12** — `.github/workflows/ci.yml`, `release.yml` 작성.
13. **Step 13** — 최종 점검: §13의 모든 항목 통과.

각 Step 완료 후 반드시 `cargo build`와 (해당 단계에 테스트가 있다면) `cargo test`를 실행해 통과를 확인하라. 다음 Step으로 넘어가기 전에 반드시 통과시켜라.

---

## 13. 완료 정의 (Definition of Done)

다음을 **모두** 통과해야 "구현 완료"라고 보고할 수 있다:

- [ ] `cargo build --release`가 경고 없이 통과한다.
- [ ] `cargo fmt --all -- --check`가 통과한다.
- [ ] `cargo clippy --all-targets --all-features -- -D warnings`가 통과한다.
- [ ] `cargo test --all`이 통과한다 (네트워크 없이).
- [ ] `target/release/gitboost --version`이 `gitboost 0.1.0`을 출력한다.
- [ ] `target/release/gitboost --help`가 `create`, `login`, `logout`, `whoami` 4개 서브커맨드를 모두 표시한다.
- [ ] `target/release/gitboost create` (인자 없이) 실행 시 종료 코드 2.
- [ ] `target/release/gitboost create "bad name"` 실행 시 0이 아닌 종료 코드와 명확한 에러 메시지.
- [ ] §3의 디렉토리 구조가 정확히 일치한다 (누락 파일/모듈 없음).
- [ ] 6개 라이선스 템플릿 파일이 모두 존재하고 비어있지 않다.
- [ ] `.github/workflows/ci.yml`과 `release.yml`이 §7, §8의 사양과 일치한다.
- [ ] README.md에 설치/사용/명령어 섹션이 모두 있다.
- [ ] 코드 어디에도 `println!`로 토큰을 출력하는 부분이 없다 (로그 포함).

---

## 14. 자주 발생하는 함정 (피해야 할 것)

1. **`reqwest`에 `default-features = true`** → OpenSSL 빌드 의존성 폭발. 반드시 `default-features = false` + `rustls-tls`.
2. **`tokio::main` 매크로 사용 후 exit code 처리 누락** → §5처럼 직접 runtime build → block_on → exit code로 변환.
3. **clap 인자에 토큰을 받기** → 절대 금지. 토큰은 stdin/keyring/Device Flow로만 입력.
4. **GitHub `gitignore/templates/{name}` API의 대소문자 불일치** → `Rust`는 되지만 `rust`는 404. §4.8.2 매핑 테이블 필수.
5. **GPG 서명이 사용자 환경에 강제된 경우 commit 실패** → `git -c commit.gpgsign=false commit ...`로 우회.
6. **Windows에서 인라인 credential helper 셸 문법 미동작** → `.bat` 파일 방식으로 분기 (§4.9.2).
7. **`git init --initial-branch=main` 미지원 (구버전 git)** → `git init` 후 `git symbolic-ref HEAD refs/heads/main` 폴백.
8. **Device Flow polling 중 응답 헤더 파싱 누락** → GitHub은 종종 `interval`을 응답 본문이 아닌 새로 받은 응답에서 갱신한다. 매 응답마다 `interval` 갱신.
9. **keyring 크레이트의 Linux secret-service 의존** → CI/headless 환경에서 실패할 수 있음. 단위 테스트에서는 keyring 직접 호출을 피하고 모듈 인터페이스에 mock을 끼울 수 있도록 설계.
10. **테스트가 실제 keyring/네트워크에 의존** → CI가 깨진다. 통합 테스트는 `--ignored` 처리, 단위 테스트는 의존성 차단.

---

## 15. 마지막 지시

이 문서를 끝까지 읽었으면, 이제 **§12의 Step 1부터 순차적으로 시작**한다. 어떤 Step에서도 멈추지 말고, 컴파일/테스트 에러는 그 자리에서 직접 해결한다. 모든 Step이 끝나고 §13의 체크리스트가 전부 통과하면 그때 한 번만 "GitBoost v0.1.0 구현 완료" 메시지를 출력하고 종료하라.

시작하라.
