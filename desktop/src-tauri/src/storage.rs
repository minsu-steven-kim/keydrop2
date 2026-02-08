use rusqlite::{Connection, Result as SqliteResult};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Vault not found")]
    VaultNotFound,

    #[error("Failed to get data directory")]
    NoDataDir,
}

pub type Result<T> = std::result::Result<T, StorageError>;

/// Local storage manager using SQLite
pub struct Storage {
    conn: Connection,
}

impl Storage {
    /// Open or create the storage database
    pub fn open() -> Result<Self> {
        let db_path = Self::get_db_path()?;

        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(&db_path)?;
        let storage = Self { conn };
        storage.init_schema()?;
        Ok(storage)
    }

    /// Get the database file path
    fn get_db_path() -> Result<PathBuf> {
        let data_dir = dirs::data_dir().ok_or(StorageError::NoDataDir)?;
        Ok(data_dir.join("keydrop").join("vault.db"))
    }

    /// Initialize database schema
    fn init_schema(&self) -> Result<()> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS vault_meta (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                salt BLOB NOT NULL,
                encrypted_vault BLOB,
                version INTEGER DEFAULT 1,
                created_at INTEGER NOT NULL,
                modified_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            ",
        )?;
        Ok(())
    }

    /// Check if a vault exists
    pub fn vault_exists(&self) -> Result<bool> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM vault_meta WHERE id = 1",
            [],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    /// Create a new vault with the given salt
    pub fn create_vault(&self, salt: &[u8; 16]) -> Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        self.conn.execute(
            "INSERT OR REPLACE INTO vault_meta (id, salt, created_at, modified_at) VALUES (1, ?1, ?2, ?2)",
            rusqlite::params![salt.as_slice(), now],
        )?;
        Ok(())
    }

    /// Get the vault salt
    pub fn get_salt(&self) -> Result<[u8; 16]> {
        let salt: Vec<u8> = self
            .conn
            .query_row("SELECT salt FROM vault_meta WHERE id = 1", [], |row| {
                row.get(0)
            })
            .map_err(|_| StorageError::VaultNotFound)?;

        if salt.len() != 16 {
            return Err(StorageError::VaultNotFound);
        }

        let mut salt_array = [0u8; 16];
        salt_array.copy_from_slice(&salt);
        Ok(salt_array)
    }

    /// Save encrypted vault data
    pub fn save_vault(&self, encrypted_data: &[u8]) -> Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        self.conn.execute(
            "UPDATE vault_meta SET encrypted_vault = ?1, modified_at = ?2 WHERE id = 1",
            rusqlite::params![encrypted_data, now],
        )?;
        Ok(())
    }

    /// Load encrypted vault data
    pub fn load_vault(&self) -> Result<Vec<u8>> {
        let data: Option<Vec<u8>> = self
            .conn
            .query_row(
                "SELECT encrypted_vault FROM vault_meta WHERE id = 1",
                [],
                |row| row.get(0),
            )
            .map_err(|_| StorageError::VaultNotFound)?;

        data.ok_or(StorageError::VaultNotFound)
    }

    /// Save a setting
    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            rusqlite::params![key, value],
        )?;
        Ok(())
    }

    /// Get a setting
    #[allow(dead_code)]
    pub fn get_setting(&self, key: &str) -> Result<Option<String>> {
        let result: SqliteResult<String> = self.conn.query_row(
            "SELECT value FROM settings WHERE key = ?1",
            rusqlite::params![key],
            |row| row.get(0),
        );

        match result {
            Ok(value) => Ok(Some(value)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(StorageError::Sqlite(e)),
        }
    }

    /// Delete vault (for remote wipe/reset)
    pub fn delete_vault(&self) -> Result<()> {
        self.conn
            .execute("DELETE FROM vault_meta WHERE id = 1", [])?;
        self.conn
            .execute("DELETE FROM settings", [])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn temp_storage() -> Storage {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = Connection::open(&db_path).unwrap();
        let storage = Storage { conn };
        storage.init_schema().unwrap();
        storage
    }

    #[test]
    fn test_vault_lifecycle() {
        let storage = temp_storage();

        // Initially no vault
        assert!(!storage.vault_exists().unwrap());

        // Create vault
        let salt = [1u8; 16];
        storage.create_vault(&salt).unwrap();
        assert!(storage.vault_exists().unwrap());

        // Get salt
        let loaded_salt = storage.get_salt().unwrap();
        assert_eq!(salt, loaded_salt);

        // Save and load vault
        let data = b"encrypted vault data";
        storage.save_vault(data).unwrap();
        let loaded = storage.load_vault().unwrap();
        assert_eq!(data.as_slice(), loaded.as_slice());
    }

    #[test]
    fn test_settings() {
        let storage = temp_storage();

        // No setting initially
        assert!(storage.get_setting("test_key").unwrap().is_none());

        // Set and get
        storage.set_setting("test_key", "test_value").unwrap();
        assert_eq!(
            storage.get_setting("test_key").unwrap(),
            Some("test_value".to_string())
        );

        // Update
        storage.set_setting("test_key", "new_value").unwrap();
        assert_eq!(
            storage.get_setting("test_key").unwrap(),
            Some("new_value".to_string())
        );
    }
}
