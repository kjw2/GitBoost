use crate::error::{GitBoostError, Result};
use keyring::Entry;

fn entry() -> Result<Entry> {
    Entry::new(crate::config::KEYRING_SERVICE, crate::config::KEYRING_USER)
        .map_err(|e| GitBoostError::Auth(format!("keyring 초기화 실패: {}", e)))
}

/// keyring에 GitHub 토큰을 저장합니다.
pub fn save(token: &str) -> Result<()> {
    entry()?
        .set_password(token)
        .map_err(|e| GitBoostError::Auth(format!("keyring 저장 실패: {}", e)))
}

/// keyring에서 GitHub 토큰을 조회합니다.
///
/// 항목이 없으면 Ok(None)을 반환합니다.
pub fn load() -> Result<Option<String>> {
    match entry()?.get_password() {
        Ok(token) => Ok(Some(token)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(GitBoostError::Auth(format!("keyring 조회 실패: {}", e))),
    }
}

/// keyring에서 GitBoost 토큰을 삭제합니다.
///
/// 항목이 없어도 성공으로 처리합니다.
pub fn delete() -> Result<()> {
    match entry()?.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(GitBoostError::Auth(format!("keyring 삭제 실패: {}", e))),
    }
}
