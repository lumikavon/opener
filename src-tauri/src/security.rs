// Security module - handles secure credential storage

use keyring::Entry as KeyringEntry;
use thiserror::Error;

const SERVICE_NAME: &str = "opener";

#[derive(Error, Debug)]
pub enum SecurityError {
    #[error("Keyring error: {0}")]
    Keyring(String),
    #[error("Credential not found: {0}")]
    NotFound(String),
    #[allow(dead_code)]
    #[error("Encryption error: {0}")]
    Encryption(String),
}

pub type SecurityResult<T> = Result<T, SecurityError>;

/// Store a credential securely using the system keychain
pub fn store_credential(key: &str, value: &str) -> SecurityResult<()> {
    let entry =
        KeyringEntry::new(SERVICE_NAME, key).map_err(|e| SecurityError::Keyring(e.to_string()))?;

    entry
        .set_password(value)
        .map_err(|e| SecurityError::Keyring(e.to_string()))?;

    Ok(())
}

/// Retrieve a credential from the system keychain
pub fn get_credential(key: &str) -> SecurityResult<String> {
    let entry =
        KeyringEntry::new(SERVICE_NAME, key).map_err(|e| SecurityError::Keyring(e.to_string()))?;

    entry.get_password().map_err(|e| match e {
        keyring::Error::NoEntry => SecurityError::NotFound(key.to_string()),
        _ => SecurityError::Keyring(e.to_string()),
    })
}

/// Delete a credential from the system keychain
pub fn delete_credential(key: &str) -> SecurityResult<()> {
    let entry =
        KeyringEntry::new(SERVICE_NAME, key).map_err(|e| SecurityError::Keyring(e.to_string()))?;

    entry.delete_credential().map_err(|e| match e {
        keyring::Error::NoEntry => SecurityError::NotFound(key.to_string()),
        _ => SecurityError::Keyring(e.to_string()),
    })
}

/// Generate a unique key for SSH credentials
#[allow(dead_code)]
pub fn ssh_credential_key(entry_id: &str) -> String {
    format!("ssh_key_{}", entry_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Requires system keyring, may not work in CI
    fn test_credential_lifecycle() {
        let key = "test_credential_key";
        let value = "super_secret_password";

        // Store
        store_credential(key, value).unwrap();

        // Retrieve
        let retrieved = get_credential(key).unwrap();
        assert_eq!(retrieved, value);

        // Delete
        delete_credential(key).unwrap();

        // Verify deleted
        let result = get_credential(key);
        assert!(result.is_err());
    }
}
