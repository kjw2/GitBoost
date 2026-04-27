# CHANGELOG

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
