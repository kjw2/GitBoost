# GitBoost Progress Report

전체 파일 전수 조사 결과 — 우선순위별 이슈 목록

> 마지막 업데이트: 2026-04-27

---

## 🔴 심각 (즉시 수정 필요)

| # | 파일/경로 | 유형 | 설명 | 상태 |
|---|-----------|------|------|------|
| 1 | `gitboost/.github/workflows/` | **버그 (치명적)** | `.github/workflows/`가 `gitboost/` 하위에 있어 GitHub Actions가 **절대 실행되지 않음**. 저장소 루트(`/`)에 있어야 함 | ✅ 수정 완료 |
| 2 | `gitboost/Cargo.toml` | **잘못된 설정** | `repository = "https://github.com/your-org/gitboost"` — 플레이스홀더 URL. 실제 저장소인 `https://github.com/kjw2/GitBoost`로 교체 필요 | ✅ 수정 완료 |
| 3 | `gitboost/.github/workflows/release.yml` | **버그** | `cp README.md LICENSE "$STAGE_DIR/" \|\| true`이 `working-directory: gitboost`에서 실행되는데 `gitboost/README.md`가 없음 → 릴리즈 아카이브에 README 미포함 | ✅ 수정 완료 |

---

## 🟠 높음 (이번 릴리즈 전 수정 권장)

| # | 파일/경로 | 유형 | 설명 | 상태 |
|---|-----------|------|------|------|
| 4 | `gitboost/Cargo.toml` | **미사용 의존성** | `indicatif = "0.17"` — 코드 어디서도 사용되지 않음 (빌드 크기 낭비) | ✅ 수정 완료 |
| 5 | `gitboost/Cargo.toml` | **미사용 의존성** | `url = "2"` — 코드 어디서도 사용되지 않음 | ✅ 수정 완료 |
| 6 | `gitboost/Cargo.toml` | **미사용 dev-의존성** | `wiremock = "0.6"` — dev-deps에 있지만 테스트에서 전혀 사용되지 않음 | ✅ 수정 완료 |
| 7 | `gitboost/src/prereq.rs` | **버그 (오타)** | 에러 메시지 한국어 오타: `"downloads 에 서 설치하세요"` → `"downloads 에서 설치하세요"` (공백 이중 입력) | ✅ 이미 정상 |
| 8 | `gitboost/src/error.rs` | **버그 (오타)** | `next_action_hint()`에 이중 공백 다수: `"login\`을  실행하세요"`, `"다른  이름을"`, `"다시 시도 하세요"` | ✅ 이미 정상 |

---

## 🟡 중간 (품질 개선)

| # | 파일/경로 | 유형 | 설명 | 상태 |
|---|-----------|------|------|------|
| 9 | `gitboost/src/generator/readme.rs` | **부분 구현** | 생성되는 README.md가 `# {project_name}` 한 줄뿐. 실용적인 섹션 (설명, 설치법, 사용법, 라이선스)이 없음 | ✅ 수정 완료 |
| 10 | `gitboost/src/generator/gitignore.rs` | **데드코드** | `validate_template_name()` 함수가 항상 `Ok(())`만 반환 — 실질적으로 no-op이고 호출되지도 않음 | ✅ 수정 완료 |
| 11 | `gitboost/src/cli.rs` | **개선 사항** | `validate_repo_name()`이 `.`으로 시작하거나 끝나는 이름을 허용하지만 GitHub는 이를 금지함 (예: `.hidden`, `name.`) | ✅ 이미 정상 (GitHub 실제 허용) |
| 12 | `gitboost/CHANGELOG.md` | **날짜 오류** | v0.1.0 날짜가 `2026-04-26`으로 기재됨 (커밋 날짜 기준 2026-04-27이 맞음) | ✅ 수정 완료 |

---

## 🟢 낮음 (향후 개선)

| # | 파일/경로 | 유형 | 설명 | 상태 |
|---|-----------|------|------|------|
| 13 | `gitboost/tests/cli_smoke.rs` | **테스트 미비** | `login`, `logout`, `whoami` 명령어에 대한 통합 테스트 없음 | ✅ 현재 범위 초과 (인증 필요) |
| 14 | GitHub Secrets | **설정 누락** | `GITBOOST_GITHUB_CLIENT_ID` secret가 GitHub 저장소에 설정되어 있지 않으면 Device Flow 인증 불가 | ✅ 완료 — Client ID(`Ov23liys1cnsX7VYUfIQ`)를 `build.rs` 기본값으로 embed (secrets 우선 override 가능) |
| 15 | `gitboost/src/generator/readme.rs` | **개선 사항** | `--description` 전달 시 README에 description을 포함하지 않음 | ✅ 수정 완료 |

---

## ✅ 완료됨

| 파일/경로 | 내용 |
|-----------|------|
| `src/main.rs` | CLI 진입점, 서브커맨드 디스패치 ✅ |
| `src/orchestrator.rs` | 15단계 create 흐름 (prereq → push + 롤백) ✅ |
| `src/auth/mod.rs` | 3단계 토큰 해석 (gh CLI → keyring → Device Flow) ✅ |
| `src/auth/gh_cli.rs` | `gh auth token` 위임 ✅ |
| `src/auth/keyring_storage.rs` | OS keyring 저장/로드/삭제 ✅ |
| `src/auth/device_flow.rs` | GitHub Device Flow + Ctrl+C 처리 ✅ |
| `src/github/mod.rs` | GithubClient (GET/POST/DELETE 빌더) ✅ |
| `src/github/models.rs` | Serde 모델 + 단위 테스트 ✅ |
| `src/github/repos.rs` | get_user / create_repo / delete_repo 등 ✅ |
| `src/generator/license.rs` | MIT/Apache/BSD 임베드, GPL/MPL/Unlicense URL 플레이스홀더 ✅ |
| `src/generator/gitignore.rs` | 이름 정규화 + GitHub API 다운로드 ✅ |
| `src/generator/mod.rs` | write_files 오케스트레이션 ✅ |
| `src/git_ops/runner.rs` | GitRunner (run/run_with_env) ✅ |
| `src/git_ops/credential.rs` | Unix 인라인 헬퍼 / Windows .bat RAII ✅ |
| `src/git_ops/mod.rs` | init/stage/commit/add_remote/push ✅ |
| `src/ui/mod.rs` | step/success/fail/warn/error_block (NO_COLOR 지원) ✅ |
| `src/ui/prompt.rs` | confirm (yes_flag 우회, TTY 체크) ✅ |
| `src/prereq.rs` | git 2.28+ 버전 확인, gh CLI 선택적 확인 ✅ |
| `src/cli.rs` | clap derive CLI 정의 + 단위 테스트 ✅ |
| `src/config.rs` | 전역 상수 및 환경변수 오버라이드 ✅ |
| `src/error.rs` | GitBoostError (종료 코드 + 힌트) ✅ |
| `src/lib.rs` | 모듈 pub 익스포트 ✅ |
| `build.rs` | GITBOOST_GITHUB_CLIENT_ID 컴파일타임 주입 ✅ |
| `tests/cli_smoke.rs` | 6개 통합 테스트 (버전, 헬프, 이름 검증, 디렉토리) ✅ |
| `README.md` | 상세 사용자 문서 ✅ |
| `CHANGELOG.md` | Keep a Changelog 형식 ✅ |
| `LICENSE` | MIT 2026 ✅ |
| `.gitignore` | /target, .env 등 ✅ |
| `Cargo.toml` | LTO + strip 릴리즈 프로파일 ✅ |

---

## 릴리즈 체크리스트

- [ ] 위 🔴/🟠 이슈 전부 수정
- [ ] `cargo test --all` 통과 확인
- [ ] `.github/workflows/` 저장소 루트로 이동 후 push
- [ ] `GITBOOST_GITHUB_CLIENT_ID` GitHub Secrets 설정 안내
- [ ] `git tag v0.1.0 && git push origin v0.1.0` → release.yml 자동 실행
- [ ] GitHub Release 페이지 확인 (5개 플랫폼 바이너리 업로드)
