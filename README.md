# GitBoost

> **프로젝트 이름 하나로 로컬 Git 초기화 + GitHub 저장소 생성 + 첫 push까지 단 하나의 명령어로.**

[![CI](https://github.com/kjw2/GitBoost/actions/workflows/ci.yml/badge.svg)](https://github.com/kjw2/GitBoost/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

---

## 목차

- [소개](#소개)
- [설치](#설치)
- [빠른 시작](#빠른-시작)
- [명령어 레퍼런스](#명령어-레퍼런스)
- [인증 방식](#인증-방식)
- [라이선스 지원](#라이선스-지원)
- [사전 요구사항](#사전-요구사항)
- [보안](#보안)
- [라이선스](#라이선스)

---

## 소개

새 프로젝트를 시작할 때마다 반복하는 보일러플레이트를 제거합니다.

```
폴더 생성 → git init → README/LICENSE 작성 → GitHub 저장소 생성 → remote 등록 → 첫 push
```

이 모든 과정을 `gitboost create <이름>` 한 줄로 끝냅니다.

### 핵심 철학

| 원칙 | 의미 |
|---|---|
| **Secure by Default** | 기본 가시성 Private. 토큰은 OS 보안 저장소(keyring)에만 저장 |
| **Zero-Config** | `gh` CLI가 인증되어 있으면 추가 설정 0회 |
| **Pure Rust, Single Binary** | 외부 런타임/스크립트 의존성 없음 |
| **Idempotent & Safe** | 파괴적 동작 전 사용자 확인. 데이터 손실 금지 |
| **Fail Loud** | 에러는 다음 행동을 알 수 있도록 명확히 출력 |

---

## 설치

### 방법 1: GitHub Releases (권장)

[Releases 페이지](https://github.com/kjw2/GitBoost/releases)에서 운영체제에 맞는 바이너리를 내려받아 PATH에 추가하세요.

```bash
# Linux / macOS 예시
curl -L https://github.com/kjw2/GitBoost/releases/latest/download/gitboost-<version>-x86_64-unknown-linux-gnu.tar.gz | tar xz
sudo mv gitboost /usr/local/bin/
```

### 방법 2: 소스에서 직접 빌드

Rust toolchain (1.75+)이 필요합니다.

```bash
git clone https://github.com/kjw2/GitBoost.git
cd GitBoost/gitboost
cargo build --release
# ./target/release/gitboost 를 PATH에 추가
```

### 방법 3: cargo install (로컬)

```bash
cargo install --path ./gitboost
```

---

## 빠른 시작

```bash
# 1. GitHub 인증 (gh CLI가 있으면 자동, 없으면 Device Flow 안내)
gitboost whoami

# 2. 새 프로젝트 생성 (Private 저장소, MIT 라이선스)
gitboost create my-awesome-app

# 3. 완료! 출력 예시:
# 🎉 완료! https://github.com/alice/my-awesome-app
#    다음으로:  cd my-awesome-app
```

### 더 많은 예시

```bash
# Public 저장소 + Rust .gitignore + Apache 2.0 라이선스
gitboost create rust-cli --public -t rust -l apache-2.0

# 저장소 설명 추가
gitboost create my-app -d "내 첫 프로젝트" --public

# 원격 저장소만 만들고 push는 나중에
gitboost create my-project --no-push

# CI 환경 (모든 프롬프트 자동 동의)
gitboost create ci-project --yes
```

---

## 명령어 레퍼런스

### `gitboost create <NAME>` — 프로젝트 생성

가장 핵심적인 명령입니다. 로컬 디렉토리 생성부터 GitHub push까지 자동으로 수행합니다.

```
gitboost create <NAME> [OPTIONS]
```

**인자**

| 인자 | 설명 |
|---|---|
| `<NAME>` | 프로젝트 이름 (=저장소 이름). 영문·숫자·하이픈·언더스코어·점 허용, 최대 100자 |

**옵션**

| 옵션 | 단축 | 기본값 | 설명 |
|---|---|---|---|
| `--public` | — | false | Public 저장소로 생성 (기본은 Private) |
| `--license <ID>` | `-l` | `mit` | SPDX 라이선스 ID |
| `--template <LANG>` | `-t` | (없음) | .gitignore 언어 템플릿 |
| `--description <STR>` | `-d` | (없음) | 저장소 설명 |
| `--author <NAME>` | — | (자동) | 라이선스 저작자 이름 |
| `--no-push` | — | false | 원격 저장소 생성 후 push 생략 |
| `--yes` / `-y` | `-y` | false | 모든 확인 프롬프트 자동 동의 (CI용) |
| `--verbose` / `-v` | `-v` | false | 디버그 로그 출력 |

**지원 라이선스 (`--license`)**

| ID | 설명 | 임베드 여부 |
|---|---|---|
| `mit` | MIT License | ✅ 바이너리 임베드 |
| `apache-2.0` | Apache License 2.0 | ✅ 바이너리 임베드 |
| `bsd-3-clause` | BSD 3-Clause License | ✅ 바이너리 임베드 |
| `gpl-3.0` | GNU GPL v3.0 | ⚠️ 안내 파일 생성 → [다운로드](https://www.gnu.org/licenses/gpl-3.0.txt) |
| `mpl-2.0` | Mozilla Public License 2.0 | ⚠️ 안내 파일 생성 → [다운로드](https://www.mozilla.org/media/MPL/2.0/index.txt) |
| `unlicense` | The Unlicense | ⚠️ 안내 파일 생성 → [다운로드](https://unlicense.org/UNLICENSE) |
| `none` | 라이선스 파일 생성 안 함 | — |

> GPL-3.0 / MPL-2.0 / Unlicense를 선택하면 다운로드 URL이 담긴 안내 파일이 생성됩니다. 해당 URL에서 전문을 내려받아 LICENSE 파일에 붙여넣으세요.

**지원 .gitignore 템플릿 (`--template`)**

`rust`, `node`, `python`, `go`, `java`, `c`, `cpp`, `csharp`, `ruby`, `php`, `swift`, `kotlin`, `scala`, `dart`, `haskell`, `elixir`, `lua`, `perl`, `unity`, `unreal`, `godot`, `android` 등 GitHub 공식 템플릿 이름 사용 가능.

---

### `gitboost login` — 명시적 로그인

GitHub Device Flow를 강제 실행합니다. `gh` CLI나 keyring 토큰과 무관하게 새 토큰을 발급받아 keyring에 저장합니다.

```bash
gitboost login
# 브라우저가 자동으로 열리며, 표시된 코드를 입력하면 인증 완료
```

---

### `gitboost logout` — 로그아웃

keyring에 저장된 GitBoost 토큰을 삭제합니다. `gh` CLI 토큰에는 영향을 주지 않습니다.

```bash
gitboost logout
```

---

### `gitboost whoami` — 사용자 확인

현재 인증된 GitHub 사용자 정보와 인증 방식을 출력합니다.

```bash
gitboost whoami
# GitHub 사용자: alice (gh CLI 위임)
#   이름: Alice
#   이메일: alice@example.com
```

---

## 인증 방식

GitBoost는 3단계 인증을 순서대로 시도합니다. 상위 단계가 성공하면 하위 단계는 건너뜁니다.

### Level 1: gh CLI 위임 (Zero-Config)

[GitHub CLI (`gh`)](https://cli.github.com/)가 설치되어 인증된 상태라면 `gh auth token`으로 토큰을 자동으로 가져옵니다. **추가 설정 불필요.**

```bash
# gh CLI 설치 및 인증
brew install gh   # macOS
gh auth login
```

### Level 2: OS Keyring (재방문 사용자)

이전에 `gitboost login`으로 저장한 토큰을 OS 보안 저장소(macOS Keychain, Windows Credential Manager, Linux secret-service)에서 자동으로 읽어옵니다.

### Level 3: GitHub Device Flow (첫 사용자)

위 두 단계가 모두 실패하면 브라우저 기반 Device Flow를 실행합니다.

1. 터미널에 URL과 코드가 표시됩니다
2. 브라우저가 자동으로 열립니다 (실패하면 직접 URL 접속)
3. 코드를 입력하고 승인하면 토큰이 keyring에 저장됩니다

> **보안 참고**: `GITBOOST_GITHUB_CLIENT_ID` 환경변수가 설정되지 않으면 Device Flow를 사용할 수 없습니다. 이 경우 `gh` CLI 인증(Level 1)을 사용하세요.

---

## 사전 요구사항

| 도구 | 필수/선택 | 최소 버전 |
|---|---|---|
| `git` | **필수** | 2.28+ |
| `gh` CLI | 선택 (Level 1 인증) | 최신 권장 |
| 인터넷 연결 | **필수** | — |

---

## 실행 흐름

`gitboost create my-app` 실행 시 내부적으로 다음 단계가 수행됩니다:

```
▸ 사전 요구사항 확인 (git 2.28+)           ✓
▸ GitHub 인증 확인 (gh CLI 위임)            ✓ logged in as alice
▸ 디렉토리 검사: ./my-app                   ✓
▸ 원격 저장소 이름 충돌 검사                ✓
▸ 파일 생성 (README, LICENSE, .gitignore)  ✓
▸ Git 초기화                               ✓
▸ 초기 커밋 생성                           ✓
▸ GitHub 저장소 생성 (private)             ✓
▸ origin 등록                              ✓
▸ 첫 push                                 ✓

🎉 완료! https://github.com/alice/my-app
   다음으로:  cd my-app
```

---

## 보안

- **토큰 저장**: OS 보안 저장소(keyring)에만 저장. 평문 파일·환경변수 저장 없음
- **토큰 로깅 없음**: `--verbose` 모드에서도 토큰은 `***`로 마스킹
- **최소 권한**: GitHub OAuth scope는 `repo`만 요청
- **임시 credential helper**: `git push` 시 토큰은 자식 프로세스에만 임시 전달되며 명령 완료 즉시 소멸
- **Private by Default**: `--public` 플래그 없이는 모든 저장소가 Private

---

## 라이선스

이 프로젝트는 [MIT License](LICENSE)로 배포됩니다.

Copyright (c) 2026 GitBoost Contributors