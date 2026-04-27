# GitBoost — 프로젝트 기획서 (v1.0)

> **Git/GitHub 자동화 CLI** · 프로젝트 이름 하나로 로컬 Git 초기화 + GitHub 원격 저장소 생성 + 연동까지 한 번에.

---

## 1. 개요 (Overview)

### 1.1. 비전 (Vision)
개발자가 새 프로젝트를 시작할 때마다 반복하는 보일러플레이트(폴더 생성 → `git init` → README/LICENSE/.gitignore 작성 → GitHub 저장소 생성 → remote 등록 → 첫 push)를 **단 하나의 명령어**로 끝내는 도구.

### 1.2. 핵심 철학 (Core Philosophy)
| 원칙 | 의미 |
| :--- | :--- |
| **Secure by Default** | 모든 저장소는 기본적으로 Private. 토큰은 OS 보안 저장소에만 저장. |
| **Zero-Config** | `gh` CLI가 이미 설치·인증되어 있으면 추가 인증 절차 0회. |
| **Pure Rust, Single Binary** | 외부 런타임/스크립트 의존성 없음. 단일 바이너리로 배포. |
| **Idempotent & Safe** | 동일 명령을 재실행해도 데이터 손실 없음. 파괴적 동작 전 사용자 확인. |
| **Fail Loud, Fail Clear** | 모든 에러는 사용자가 다음 행동을 알 수 있도록 명확한 메시지로 출력. |

### 1.3. 비목표 (Non-Goals)
- GitHub Enterprise Server 지원 (v1.0 범위 외; v1.1+에서 검토).
- GitLab/Bitbucket 등 타 호스팅 서비스 지원.
- Git 기본 기능(commit, branch, merge 등)의 래핑 — 그건 그냥 `git`을 쓰면 됨.
- GUI 제공.

---

## 2. 기술 스택 (Technical Stack)

| 영역 | 라이브러리 / 도구 | 버전 (최소) | 비고 |
| :--- | :--- | :--- | :--- |
| 언어 | Rust (Stable) | 1.75+ | edition = "2021" |
| CLI 파싱 | `clap` | 4.5 | derive feature |
| 비동기 런타임 | `tokio` | 1.40 | features = `["rt-multi-thread", "macros", "process"]` |
| HTTP 클라이언트 | `reqwest` | 0.12 | features = `["json", "rustls-tls"]` (OpenSSL 의존성 회피) |
| JSON | `serde`, `serde_json` | 1.x | derive |
| 에러 처리 | `thiserror`, `anyhow` | 최신 | 라이브러리는 thiserror, 바이너리는 anyhow |
| OS 키체인 | `keyring` | 3.x | 크로스 플랫폼 |
| 로깅 | `tracing`, `tracing-subscriber` | 최신 | `--verbose` 플래그로 레벨 조정 |
| 컬러 출력 | `owo-colors` | 4.x | NO_COLOR 환경변수 자동 존중 |
| 진행 표시 | `indicatif` | 0.17 | spinner |
| 입력 프롬프트 | `dialoguer` | 0.11 | confirm/select |
| URL 파싱 | `url` | 2.x | |
| Git 제어 | `std::process::Command` | — | 시스템 `git` 바이너리 호출 (libgit2 미사용) |
| CI/CD | GitHub Actions | — | matrix build |
| 빌드 옵션 | `Cargo.toml` `[profile.release]` | — | `lto = true`, `codegen-units = 1`, `strip = true` |

---

## 3. 시스템 사전 요구사항 (Prerequisites)

런타임에 다음을 검사하고, 없으면 명확한 안내 메시지와 함께 종료한다.

| 도구 | 필수/선택 | 검사 방법 | 부재 시 동작 |
| :--- | :--- | :--- | :--- |
| `git` (>= 2.28, `init.defaultBranch` 지원) | **필수** | `git --version` | 에러 종료. 설치 안내 출력. |
| `gh` (GitHub CLI) | 선택 | `gh --version` | Level 1 인증 스킵, Level 2/3로 진행. |
| 인터넷 연결 | **필수** | API 호출 시 검사 | 네트워크 에러 발생 시 친절한 메시지. |

> **Git 2.28 미만**: `--initial-branch=main` 플래그가 없으므로 `git init` 후 `git symbolic-ref HEAD refs/heads/main`으로 폴백한다.

---

## 4. 핵심 기능 상세 설계

### 4.1. 스마트 하이브리드 인증 (Auth Flow)

#### 4.1.1. 인증 우선순위
세 단계를 **위에서 아래로** 순차 시도하며, 성공 시 즉시 종료한다.

**Level 1 — `gh` CLI 위임 (Zero-Config Path)**
1. `gh --version`이 성공하는지 확인.
2. `gh auth token`을 실행해 stdout에서 토큰을 읽는다.
3. 토큰이 비어있지 않고, `GET /user` 호출 시 200 OK이면 성공.
4. **이 토큰은 keyring에 저장하지 않는다** (gh가 관리하는 토큰의 라이프사이클을 침범하지 않기 위함).

**Level 2 — Keyring 조회 (Returning User Path)**
1. service = `"gitboost"`, user = `"github_token"`로 keyring에서 토큰 조회.
2. 존재하면 `GET /user`로 유효성 검증.
3. 401/403 응답 시 keyring에서 해당 항목 삭제 후 Level 3로 진행.

**Level 3 — GitHub Device Flow (First-Time User Path)**
1. `POST https://github.com/login/device/code`
   - `client_id`: 컴파일타임 환경변수 `GITBOOST_GITHUB_CLIENT_ID`로 주입.
   - `scope`: `repo` (저장소 생성·관리에 필요한 최소 스코프).
2. 사용자에게 `verification_uri`와 `user_code`를 표시. 가능하면 브라우저 자동 오픈(`open`/`xdg-open`/`start`); 실패해도 진행 가능.
3. `interval`초 간격으로 `POST https://github.com/login/oauth/access_token`을 폴링.
4. 응답 처리:
   - `authorization_pending` → 계속 폴링.
   - `slow_down` → interval에 5초 추가.
   - `expired_token` → 에러 메시지 후 종료, 사용자가 재시도하도록 안내.
   - `access_denied` → 사용자가 거부했음을 알리고 종료.
   - `access_token` 수신 → keyring에 저장, 성공.

#### 4.1.2. 보안 요건
- 토큰을 **로그에 절대 출력하지 않는다**. `Debug` 구현 시 마스킹.
- 토큰을 환경변수로 자식 프로세스에 전달하지 않는다 (단, `git push` 시 임시 credential helper로 사용하는 경우는 예외).
- `--logout` 명령으로 keyring의 토큰을 삭제할 수 있어야 한다.

### 4.2. 저장소 및 파일 생성 정책

| 항목 | 기본값 | 옵션 / 동작 |
| :--- | :--- | :--- |
| 가시성 | **Private** | `--public` 사용 시 Public |
| 라이선스 | **MIT** (`LICENSE` 파일 생성, 연도/이름 치환) | `--license <SPDX_ID>` (mit, apache-2.0, gpl-3.0, bsd-3-clause, mpl-2.0, unlicense, none) |
| README | `# <프로젝트이름>\n` 1줄 | 항상 생성. 이미 있으면 덮어쓰지 않음. |
| .gitignore | 생성 안 함 | `--template <LANG>` 사용 시 GitHub gitignore API에서 다운로드 (rust, node, python, go, java 등) |
| 기본 브랜치 | `main` | 변경 옵션 없음 (v1.0). |
| 첫 커밋 메시지 | `chore: initial commit by GitBoost` | 변경 옵션 없음 (v1.0). |
| 커밋 작성자 | `git config user.name/email`을 사용. 비어있으면 GitHub API의 `name`/`email`로 폴백. | — |

#### 4.2.1. 라이선스 본문 생성 규칙
- MIT/BSD-3-Clause는 **연도와 저작자**가 필요 → 저작자는 다음 우선순위로 결정:
  1. `--author "<NAME>"` 옵션
  2. `git config user.name`
  3. GitHub API `/user` 응답의 `name` (없으면 `login`)
- 라이선스 본문은 **바이너리에 임베드**된 템플릿(`include_str!`)을 사용. 네트워크 의존성 제거.
- `--license none`이면 LICENSE 파일을 만들지 않는다.

#### 4.2.2. .gitignore 다운로드 규칙
- 엔드포인트: `GET https://api.github.com/gitignore/templates/{name}` (대소문자 정확히, e.g. `Rust`, `Node`, `Python`).
- 사용자가 입력한 값은 정규화: `rust → Rust`, `node → Node` 등 (lowercase 매핑 테이블 유지).
- 응답의 `source` 필드를 `.gitignore`로 저장.
- 네트워크 실패 시 경고 출력하고 `.gitignore` 없이 진행 (치명적 에러 아님).

### 4.3. 디렉토리 및 충돌 처리 정책

명령 실행 위치는 **현재 작업 디렉토리(CWD)**. 동작 순서:

1. `<CWD>/<NAME>` 경로 확인.
   - 존재하지 않으면 → 생성하고 진입.
   - 존재하지만 비어있으면 → 진입.
   - 존재하고 파일이 있으면 → **에러 종료**. `--force` 옵션은 v1.0 미지원 (안전성 우선).
2. GitHub에서 동일 이름 저장소 존재 검사 (`GET /repos/{owner}/{name}`).
   - 존재하면 → 에러 종료. 이름 변경을 안내.
3. 모든 사전 검사 통과 후에야 파일 생성을 시작.

### 4.4. Git 작업 흐름 (git_ops)

순서를 **엄격히** 지킨다. 각 단계 실패 시 명확한 에러를 던지고 롤백 가능한 부분은 롤백한다.

```
1. git init --initial-branch=main          (실패 시 init 후 symbolic-ref 폴백)
2. README.md, LICENSE, .gitignore 등 파일 생성
3. git add .
4. git -c commit.gpgsign=false commit -m "chore: initial commit by GitBoost"
   (사용자 환경의 GPG 설정과 무관하게 동작하도록 gpgsign=false 강제)
5. GitHub API: POST /user/repos  (private/public, description 등)
6. git remote add origin <clone_url>       (HTTPS 사용)
7. git push -u origin main
   (인증: 임시 credential helper로 토큰 주입. push 후 helper 자동 정리)
```

#### 4.4.1. Push 인증 메커니즘
`git push` 시 토큰을 어떻게 전달할지가 핵심 보안 포인트.

**선택된 방식**: `git -c credential.helper=` 로 기존 헬퍼를 무시하고, `-c credential.helper="!f() { echo username=x-access-token; echo password=$GITBOOST_TOKEN; }; f"`처럼 임시 인라인 헬퍼를 주입한다. 토큰은 환경변수로 자식 프로세스 한정 전달, 명령 종료 시 자연 소멸.

> Windows의 경우 `cmd.exe` 셸 문법 차이로 인라인 헬퍼가 동작하지 않을 수 있다. 이때는 임시 디렉토리에 헬퍼 스크립트(`.bat`)를 작성하고 절대 경로로 지정 후, 명령 완료 즉시 삭제한다 (try/finally로 보장).

#### 4.4.2. 롤백 정책
- GitHub 저장소 생성에는 **성공**했으나 `git push`에 실패한 경우:
  - 사용자에게 상황을 설명하고, 원격 저장소를 삭제할지 묻는다 (`dialoguer::Confirm`, 기본 No).
  - 사용자가 동의하면 `DELETE /repos/{owner}/{repo}` 호출.
  - 비대화형(`--yes` 또는 stdin이 TTY가 아님) 환경에서는 삭제하지 않고 메시지만 출력.
- 로컬 디렉토리 생성/파일 작성에 실패한 경우: GitHub 호출 전 단계이므로 단순 종료.

### 4.5. CLI 명령어 디자인

```
gitboost <COMMAND> [OPTIONS]
```

| 명령 | 설명 |
| :--- | :--- |
| `create <NAME>` | 새 프로젝트 생성. 핵심 명령. |
| `login` | 명시적 인증 트리거 (Device Flow 강제 실행). |
| `logout` | keyring에서 GitBoost 토큰 삭제. |
| `whoami` | 현재 인증된 GitHub 사용자 표시. |
| `--version` / `-V` | 버전 출력. |
| `--help` / `-h` | 도움말. |

#### 4.5.1. `create` 옵션 전체 목록
| 옵션 | 단축 | 타입 | 기본값 | 설명 |
| :--- | :--- | :--- | :--- | :--- |
| `--public` | — | flag | false | Public 저장소 생성 |
| `--license <ID>` | `-l` | string | `mit` | SPDX ID. `none`도 가능. |
| `--template <LANG>` | `-t` | string | (없음) | .gitignore 템플릿 |
| `--description <STR>` | `-d` | string | (없음) | 저장소 설명 |
| `--author <NAME>` | — | string | (자동) | 라이선스 저작자 |
| `--no-push` | — | flag | false | 원격 생성 후 push 생략 |
| `--yes` | `-y` | flag | false | 모든 확인 프롬프트에 자동 동의 (CI용) |
| `--verbose` | `-v` | flag | false | 디버그 로그 출력 |

#### 4.5.2. 사용 예시
```bash
gitboost create my-app                                # Private + MIT
gitboost create my-app --public                       # Public + MIT
gitboost create rust-cli -t rust -l apache-2.0        # Rust .gitignore + Apache 2.0
gitboost create demo --no-push                        # 원격은 만들되 push는 안 함
gitboost create my-app -d "내 첫 프로젝트" --public
```

### 4.6. 종료 코드 (Exit Codes)
| 코드 | 의미 |
| :--- | :--- |
| 0 | 성공 |
| 1 | 일반 에러 (예측되지 않은 실패) |
| 2 | CLI 인자 파싱 실패 (clap 기본) |
| 10 | 사전 요구사항 미충족 (git 미설치 등) |
| 11 | 인증 실패 |
| 12 | GitHub API 에러 (권한 부족, 이름 충돌 등) |
| 13 | 로컬 파일시스템 에러 (디렉토리 충돌 등) |
| 14 | Git 명령 실패 |
| 15 | 네트워크 에러 |

---

## 5. 아키텍처 (Module Structure)

```
gitboost/
├── Cargo.toml
├── Cargo.lock
├── README.md
├── LICENSE
├── build.rs                       # GITBOOST_GITHUB_CLIENT_ID 검증
├── .github/
│   └── workflows/
│       ├── ci.yml
│       └── release.yml
├── tests/
│   ├── cli_smoke.rs               # assert_cmd 기반 통합 테스트
│   └── helpers/mod.rs
└── src/
    ├── main.rs                    # 엔트리포인트, tokio runtime, 에러 → exit code 매핑
    ├── lib.rs                     # 모든 모듈 re-export
    ├── cli.rs                     # clap 정의 (Args/Subcommand)
    ├── error.rs                   # GitBoostError (thiserror) + ExitCode 매핑
    ├── config.rs                  # 상수 (CLIENT_ID, USER_AGENT, KEYRING_SERVICE 등)
    ├── auth/
    │   ├── mod.rs                 # AuthProvider 트레이트, resolve_token() 진입점
    │   ├── gh_cli.rs              # `gh auth token` 호출
    │   ├── keyring_storage.rs     # keyring 래퍼 (저장/조회/삭제)
    │   └── device_flow.rs         # Device Flow 구현 + 폴링 루프
    ├── github/
    │   ├── mod.rs                 # GithubClient (reqwest 래퍼)
    │   ├── models.rs              # Repo, User, ApiError 등 serde 모델
    │   └── repos.rs               # create_repo, get_repo, delete_repo, get_user
    ├── generator/
    │   ├── mod.rs                 # write_files() 오케스트레이션
    │   ├── readme.rs              # README 생성
    │   ├── license.rs             # 라이선스 템플릿 + 치환 (include_str!)
    │   ├── gitignore.rs           # gitignore API 호출 + 정규화 매핑
    │   └── templates/             # 라이선스 원본 텍스트
    │       ├── mit.txt
    │       ├── apache-2.0.txt
    │       ├── gpl-3.0.txt
    │       ├── bsd-3-clause.txt
    │       ├── mpl-2.0.txt
    │       └── unlicense.txt
    ├── git_ops/
    │   ├── mod.rs                 # init → commit → remote → push 오케스트레이션
    │   ├── runner.rs              # Command 실행 헬퍼 (stdout/stderr 캡처, 에러 변환)
    │   └── credential.rs          # 임시 credential helper 주입 + 정리
    ├── ui/
    │   ├── mod.rs                 # 출력 포맷팅 (성공/실패/스피너)
    │   └── prompt.rs              # dialoguer 래퍼 (--yes 존중)
    └── prereq.rs                  # git/gh 버전 검사
```

### 5.1. 모듈 간 의존성 규칙
- `cli` → `auth`, `github`, `generator`, `git_ops`, `prereq`, `ui` 호출 가능.
- `auth`, `github`, `generator`, `git_ops`는 **서로를 직접 참조하지 않는다**. 오케스트레이션은 `cli`/`main`이 담당.
- `error`, `config`, `ui`는 모든 모듈이 사용.

---

## 6. 에러 처리 전략

### 6.1. `GitBoostError` (thiserror)
```rust
#[derive(thiserror::Error, Debug)]
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
```

### 6.2. 사용자에게 보여지는 메시지 규칙
- 모든 에러는 `red bold`로 시작하는 한 줄 요약 + 들여쓴 상세 설명 + (있으면) **다음 행동 제안**.
- 예시:
  ```
  ✗ 인증 실패
    Device Flow 토큰 발급에 실패했습니다 (expired_token).
    → 다시 시도하려면 `gitboost login`을 실행하세요.
  ```

### 6.3. 패닉 정책
- `unwrap()`/`expect()`는 **테스트 코드와 컴파일타임 상수 검증에만** 허용.
- 모든 런타임 실패는 `Result`로 전파.

---

## 7. 사용자 경험 (UX)

### 7.1. 표준 실행 흐름 출력 예시
```
$ gitboost create my-cool-app -t rust

▸ 사전 요구사항 확인...                   ✓
▸ GitHub 인증 (gh CLI 위임)...            ✓ logged in as alice
▸ 디렉토리 검사: ./my-cool-app             ✓
▸ 원격 저장소 이름 충돌 검사...            ✓
▸ 파일 생성 (README, LICENSE, .gitignore) ✓
▸ Git 초기화 + 초기 커밋...                ✓
▸ GitHub 저장소 생성 (private)...          ✓
▸ origin 등록 + 첫 push...                ✓

🎉 완료! https://github.com/alice/my-cool-app
   다음으로:  cd my-cool-app
```

### 7.2. 컬러 정책
- 성공: green, 실패: red, 정보: cyan, 경고: yellow.
- `NO_COLOR` 환경변수 또는 stdout이 TTY가 아니면 컬러/스피너 비활성화.

### 7.3. 로깅
- 기본: `INFO` 이상의 사용자 메시지만.
- `--verbose`: `DEBUG` (HTTP 요청/응답 헤더, Command 실행 세부정보 — **토큰 마스킹 필수**).
- `RUST_LOG` 환경변수 사용 시 우선 적용.

---

## 8. 테스트 전략

### 8.1. 단위 테스트
- `generator/license.rs`: 연도/저작자 치환 결과 검증.
- `generator/gitignore.rs`: 사용자 입력 → 정규화 매핑 검증.
- `auth/keyring_storage.rs`: mock keyring (feature flag로 분리) 사용한 라운드트립.
- `github/models.rs`: 실제 API 응답 fixture로 역직렬화 테스트.

### 8.2. 통합 테스트 (`tests/`)
- `assert_cmd` + `predicates`로 CLI 종단 검증:
  - `gitboost --version` 출력 형식.
  - `gitboost create` 인자 누락 시 종료 코드 2.
  - `gitboost create existing-dir` 시 종료 코드 13.
- 네트워크가 필요한 테스트는 `#[ignore]` 마킹, `cargo test -- --ignored`로만 실행.

### 8.3. 모의 GitHub API
- `wiremock` 크레이트로 로컬 mock 서버 띄워 GithubClient 테스트.
- `GITHUB_API_BASE` 환경변수로 베이스 URL 오버라이드 가능하게 설계 (테스트 한정).

---

## 9. CI/CD 및 배포 전략

### 9.1. CI (`.github/workflows/ci.yml`)
모든 PR과 push에서 실행:
1. `cargo fmt --check`
2. `cargo clippy --all-targets -- -D warnings`
3. `cargo test`
4. (선택) `cargo audit`로 취약점 스캔.

### 9.2. CD (`.github/workflows/release.yml`)
태그 `v*` push 시 트리거. Matrix:

| OS | Target | 비고 |
| :--- | :--- | :--- |
| ubuntu-latest | x86_64-unknown-linux-gnu | native |
| ubuntu-latest | aarch64-unknown-linux-gnu | `cross` 사용 |
| macos-latest | x86_64-apple-darwin | native |
| macos-latest | aarch64-apple-darwin | native |
| windows-latest | x86_64-pc-windows-msvc | native, `.exe` 접미사 |

**산출물 명명 규칙**: `gitboost-<version>-<target>.tar.gz` (Windows는 `.zip`). SHA256 체크섬도 함께 업로드.

**시크릿 주입**:
- `GITBOOST_GITHUB_CLIENT_ID`는 GitHub Secrets에 보관, build job에서 환경변수로 주입.
- 태그 푸시 시에만 production client_id 사용. PR/CI에서는 빈 값 허용 (인증 관련 통합 테스트 스킵).

### 9.3. 버전 관리
- SemVer 준수. `Cargo.toml`의 version과 git 태그가 일치해야 release job이 동작.
- CHANGELOG.md를 Keep a Changelog 형식으로 유지.

---

## 10. 향후 확장성 (Roadmap)

### v1.1
- `gitboost sync <DIR>`: 기존 로컬 폴더를 GitHub에 사후 연동.
- 사용자 정의 라이선스 템플릿 (`~/.config/gitboost/licenses/*.txt`).

### v1.2
- 사용자 정의 프로젝트 스캐폴드 템플릿 (`gitboost create my-app --scaffold rust-cli`).
- 조직(`org`) 소유 저장소 생성 (`--org <NAME>`).

### v1.3
- GitHub Enterprise Server 지원 (`--host <URL>`).
- SSH remote 옵션 (`--ssh`).

---

## 11. 부록: 외부 API 엔드포인트 요약

| 용도 | 메서드 | URL | 인증 | 비고 |
| :--- | :--- | :--- | :--- | :--- |
| 사용자 정보 | GET | `https://api.github.com/user` | Bearer | 토큰 검증 |
| 저장소 생성 | POST | `https://api.github.com/user/repos` | Bearer | body: name, private, description, auto_init=false |
| 저장소 조회 | GET | `https://api.github.com/repos/{owner}/{repo}` | Bearer | 충돌 검사 |
| 저장소 삭제 | DELETE | `https://api.github.com/repos/{owner}/{repo}` | Bearer | 롤백용 |
| .gitignore 템플릿 | GET | `https://api.github.com/gitignore/templates/{name}` | (선택) | 인증 없이도 호출 가능 |
| Device code 발급 | POST | `https://github.com/login/device/code` | client_id | scope=repo |
| Access token 폴링 | POST | `https://github.com/login/oauth/access_token` | client_id + device_code | grant_type=urn:ietf:params:oauth:grant-type:device_code |

**모든 GitHub API 호출 공통 헤더**:
- `Accept: application/vnd.github+json`
- `User-Agent: gitboost/<version>`
- `X-GitHub-Api-Version: 2022-11-28`
