# CHANGELOG

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.4] - 2026-04-27

### Fixed
- **명령어마다 재인증 요구**: `resolve_token`이 매 호출마다 GitHub API로 토큰을 검증하첰고, 검증 실패 시 keyring을 삭제해 Device Flow를 반복 요구하던 문제 시정  
  → `resolve_token`에서 네트워크 검증 완전 제거. keyring/gh-CLI 토큰을 네트워크 호출 없이 즐시 사용
- **유효하지 않은 토큰 처리**: GitHub API가 401을 반환하면 keyring을 자동 정리하고 `gitboost login` 안내 메시지 표시

## [0.1.3] - 2026-04-27

### Fixed
- **push 실패 (Windows)**: `.bat` credential helper가 git 내부 bash에서 경로 파싱 오류 발생하는 문제 수정  
  → `git remote set-url`로 `x-access-token:{token}@` 임시 주입 방식으로 교체 (Unix/Windows 공통)
- **이중 로그인**: `verify_token` 네트워크 오류 시 keyring 토큰을 삭제하여 불필요한 재인증 유도하던 문제 수정  
  → 401 Unauthorized 응답일 때만 토큰 삭제, 네트워크 오류 시 기존 토큰 유지

## [0.1.2] - 2026-04-27

### Fixed
- `build.rs`: GHA에서 secret 미설정 시 `GITBOOST_GITHUB_CLIENT_ID`가 빈 문자열로 전달되는 경우에도 기본 Client ID를 사용하도록 수정 (`filter(|s| !s.is_empty())` 추가)

## [0.1.1] - 2026-04-27

### Fixed
- GitHub Device Flow Client ID를 `build.rs` 기본값으로 embed — `GITBOOST_GITHUB_CLIENT_ID` 환경변수 없이도 즉시 동작
- Release 워크플로우에 Client ID 주석 보강 (secrets 우선 override 가능)

## [0.1.0] - 2026-04-27

### Added
- `create` command: 로컬 디렉토리 생성, Git 초기화, README/LICENSE/.gitignore 작성, GitHub 원격 저장소 생성, remote 등록 + 첫 push 자동화
- `login` command: GitHub Device Flow를 통한 명시적 인증
- `logout` command: keyring에 저장된 GitBoost 토큰 삭제
- `whoami` command: 현재 인증된 GitHub 사용자 정보 출력
- 스마트 하이브리드 인증 (gh CLI → keyring → Device Flow)
- 6개 SPDX 라이선스 지원 (MIT, Apache-2.0, GPL-3.0, BSD-3-Clause, MPL-2.0, Unlicense)
- GitHub gitignore 템플릿 다운로드 (`--template` 옵션)
- Secure by Default: 기본 가시성 Private, 토큰은 OS keyring에만 저장
- Cross-platform 지원 (Linux, macOS, Windows)
- NO_COLOR 환경변수 존중
