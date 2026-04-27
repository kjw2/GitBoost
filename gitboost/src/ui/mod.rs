pub mod prompt;

use crate::error::GitBoostError;
use owo_colors::OwoColorize;

/// 작업 단계 시작을 알리는 메시지 출력 (cyan)
pub fn step(label: &str) {
    if is_color_enabled() {
        eprintln!("{} {}", "▸".cyan(), label);
    } else {
        eprintln!("▸ {}", label);
    }
}

/// 성공 메시지 출력 (green)
pub fn success(label: &str) {
    if is_color_enabled() {
        eprintln!("{} {}", "✓".green(), label);
    } else {
        eprintln!("✓ {}", label);
    }
}

/// 실패 메시지 출력 (red)
pub fn fail(label: &str) {
    if is_color_enabled() {
        eprintln!("{} {}", "✗".red(), label);
    } else {
        eprintln!("✗ {}", label);
    }
}

/// 일반 정보 메시지 출력
pub fn info(msg: &str) {
    eprintln!("  {}", msg);
}

/// 경고 메시지 출력 (yellow)
pub fn warn(msg: &str) {
    if is_color_enabled() {
        eprintln!("  {} {}", "⚠".yellow(), msg);
    } else {
        eprintln!("  ⚠ {}", msg);
    }
}

/// 에러 블록 출력: 에러 요약 + 힌트
pub fn error_block(err: &GitBoostError) {
    if is_color_enabled() {
        eprintln!("\n{} {}", "✗".red().bold(), err.to_string().red().bold());
    } else {
        eprintln!("\n✗ {}", err);
    }

    if let Some(hint) = err.next_action_hint() {
        eprintln!("  → {}", hint);
    }
    eprintln!();
}

/// NO_COLOR 환경변수 및 TTY 여부로 컬러 출력 가능 여부 판단
fn is_color_enabled() -> bool {
    if std::env::var("NO_COLOR").is_ok() {
        return false;
    }
    // stderr가 TTY인지 확인
    use std::io::IsTerminal;
    std::io::stderr().is_terminal()
}
