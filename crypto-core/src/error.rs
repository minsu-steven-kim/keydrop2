use thiserror::Error;

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("Key derivation failed: {0}")]
    KeyDerivation(String),

    #[error("Encryption failed: {0}")]
    Encryption(String),

    #[error("Decryption failed: {0}")]
    Decryption(String),

    #[error("Invalid key length: expected {expected}, got {got}")]
    InvalidKeyLength { expected: usize, got: usize },

    #[error("Invalid nonce length: expected {expected}, got {got}")]
    InvalidNonceLength { expected: usize, got: usize },

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Deserialization error: {0}")]
    Deserialization(String),

    #[error("Vault item not found: {0}")]
    ItemNotFound(String),

    #[error("Invalid password options: {0}")]
    InvalidPasswordOptions(String),

    #[error("Random generation failed: {0}")]
    RandomGeneration(String),
}

pub type Result<T> = std::result::Result<T, CryptoError>;
