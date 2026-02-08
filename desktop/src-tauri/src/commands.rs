use crate::state::AppState;
use crate::storage::Storage;
use crate::sync::{RemoteCommand, SyncState, SyncStatus};
use crypto_core::{
    cipher::EncryptedBlob,
    kdf::{derive_keys, derive_master_key, Salt},
    password::{generate_passphrase, generate_password, PasswordOptions},
    vault::{Vault, VaultItem},
};
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize)]
pub struct CommandError {
    pub message: String,
}

impl From<crypto_core::error::CryptoError> for CommandError {
    fn from(e: crypto_core::error::CryptoError) -> Self {
        CommandError {
            message: e.to_string(),
        }
    }
}

impl From<crate::storage::StorageError> for CommandError {
    fn from(e: crate::storage::StorageError) -> Self {
        CommandError {
            message: e.to_string(),
        }
    }
}

type CommandResult<T> = Result<T, CommandError>;

// =============================================================================
// Vault Status Commands
// =============================================================================

#[derive(Serialize)]
pub struct VaultStatus {
    pub exists: bool,
    pub unlocked: bool,
}

#[tauri::command]
pub fn get_vault_status(state: State<AppState>) -> CommandResult<VaultStatus> {
    let storage = Storage::open()?;
    Ok(VaultStatus {
        exists: storage.vault_exists()?,
        unlocked: state.is_unlocked(),
    })
}

// =============================================================================
// Vault Creation & Unlock
// =============================================================================

#[tauri::command]
pub fn create_vault(password: String, state: State<AppState>) -> CommandResult<()> {
    let storage = Storage::open()?;

    if storage.vault_exists()? {
        return Err(CommandError {
            message: "Vault already exists".to_string(),
        });
    }

    // Generate salt and derive keys
    let salt = Salt::generate()?;
    let master_key = derive_master_key(&password, &salt)?;
    let keys = derive_keys(&master_key)?;

    // Create empty vault
    let vault = Vault::new();

    // Encrypt and save
    let encrypted = vault.export(&keys.vault_key)?;
    let encrypted_bytes = serde_json::to_vec(&encrypted).map_err(|e| CommandError {
        message: e.to_string(),
    })?;

    storage.create_vault(salt.as_bytes())?;
    storage.save_vault(&encrypted_bytes)?;

    // Update state
    *state.vault.lock().unwrap() = Some(vault);
    *state.keys.lock().unwrap() = Some(keys);
    *state.salt.lock().unwrap() = Some(*salt.as_bytes());
    state.touch();

    Ok(())
}

#[tauri::command]
pub fn unlock_vault(password: String, state: State<AppState>) -> CommandResult<()> {
    let storage = Storage::open()?;

    if !storage.vault_exists()? {
        return Err(CommandError {
            message: "No vault exists".to_string(),
        });
    }

    // Load salt and encrypted vault
    let salt_bytes = storage.get_salt()?;
    let salt = Salt::from_bytes(salt_bytes);
    let encrypted_bytes = storage.load_vault()?;

    // Derive keys
    let master_key = derive_master_key(&password, &salt)?;
    let keys = derive_keys(&master_key)?;

    // Decrypt vault
    let encrypted: EncryptedBlob = serde_json::from_slice(&encrypted_bytes).map_err(|e| {
        CommandError {
            message: e.to_string(),
        }
    })?;
    let vault = Vault::import(&encrypted, &keys.vault_key)?;

    // Update state
    *state.vault.lock().unwrap() = Some(vault);
    *state.keys.lock().unwrap() = Some(keys);
    *state.salt.lock().unwrap() = Some(salt_bytes);
    state.touch();

    Ok(())
}

#[tauri::command]
pub fn lock_vault(state: State<AppState>) -> CommandResult<()> {
    state.lock();
    Ok(())
}

// =============================================================================
// Vault Item Commands
// =============================================================================

#[derive(Serialize, Deserialize)]
pub struct VaultItemDto {
    pub id: String,
    pub name: String,
    pub url: Option<String>,
    pub username: String,
    pub password: String,
    pub notes: Option<String>,
    pub category: Option<String>,
    pub favorite: bool,
    pub created_at: u64,
    pub modified_at: u64,
}

impl From<&VaultItem> for VaultItemDto {
    fn from(item: &VaultItem) -> Self {
        VaultItemDto {
            id: item.id.clone(),
            name: item.name.clone(),
            url: item.url.clone(),
            username: item.username.clone(),
            password: item.password.clone(),
            notes: item.notes.clone(),
            category: item.category.clone(),
            favorite: item.favorite,
            created_at: item.created_at,
            modified_at: item.modified_at,
        }
    }
}

impl From<VaultItemDto> for VaultItem {
    fn from(dto: VaultItemDto) -> Self {
        let mut item = VaultItem::new(&dto.name, &dto.username, &dto.password);
        item.id = dto.id;
        item.url = dto.url;
        item.notes = dto.notes;
        item.category = dto.category;
        item.favorite = dto.favorite;
        item
    }
}

fn save_vault_to_storage(state: &State<AppState>) -> CommandResult<()> {
    let vault = state.vault.lock().unwrap();
    let keys = state.keys.lock().unwrap();

    let vault = vault.as_ref().ok_or(CommandError {
        message: "Vault is locked".to_string(),
    })?;
    let keys = keys.as_ref().ok_or(CommandError {
        message: "Keys not available".to_string(),
    })?;

    let encrypted = vault.export(&keys.vault_key)?;
    let encrypted_bytes = serde_json::to_vec(&encrypted).map_err(|e| CommandError {
        message: e.to_string(),
    })?;

    let storage = Storage::open()?;
    storage.save_vault(&encrypted_bytes)?;

    Ok(())
}

#[tauri::command]
pub fn get_all_items(state: State<AppState>) -> CommandResult<Vec<VaultItemDto>> {
    state.touch();
    let vault = state.vault.lock().unwrap();
    let vault = vault.as_ref().ok_or(CommandError {
        message: "Vault is locked".to_string(),
    })?;

    Ok(vault.items.iter().map(VaultItemDto::from).collect())
}

#[tauri::command]
pub fn get_item(id: String, state: State<AppState>) -> CommandResult<Option<VaultItemDto>> {
    state.touch();
    let vault = state.vault.lock().unwrap();
    let vault = vault.as_ref().ok_or(CommandError {
        message: "Vault is locked".to_string(),
    })?;

    Ok(vault.get_item(&id).map(VaultItemDto::from))
}

#[tauri::command]
pub fn add_item(item: VaultItemDto, state: State<AppState>) -> CommandResult<String> {
    state.touch();
    let id = {
        let mut vault_guard = state.vault.lock().unwrap();
        let vault = vault_guard.as_mut().ok_or(CommandError {
            message: "Vault is locked".to_string(),
        })?;

        let vault_item: VaultItem = item.into();
        vault.add_item(vault_item)
    };

    save_vault_to_storage(&state)?;
    Ok(id)
}

#[tauri::command]
pub fn update_item(id: String, item: VaultItemDto, state: State<AppState>) -> CommandResult<()> {
    state.touch();
    {
        let mut vault_guard = state.vault.lock().unwrap();
        let vault = vault_guard.as_mut().ok_or(CommandError {
            message: "Vault is locked".to_string(),
        })?;

        let vault_item: VaultItem = item.into();
        vault.update_item(&id, vault_item)?;
    }

    save_vault_to_storage(&state)?;
    Ok(())
}

#[tauri::command]
pub fn delete_item(id: String, state: State<AppState>) -> CommandResult<()> {
    state.touch();
    {
        let mut vault_guard = state.vault.lock().unwrap();
        let vault = vault_guard.as_mut().ok_or(CommandError {
            message: "Vault is locked".to_string(),
        })?;

        vault.remove_item(&id)?;
    }

    save_vault_to_storage(&state)?;
    Ok(())
}

#[tauri::command]
pub fn search_items(query: String, state: State<AppState>) -> CommandResult<Vec<VaultItemDto>> {
    state.touch();
    let vault = state.vault.lock().unwrap();
    let vault = vault.as_ref().ok_or(CommandError {
        message: "Vault is locked".to_string(),
    })?;

    Ok(vault.search(&query).iter().map(|i| (*i).into()).collect())
}

#[tauri::command]
pub fn get_favorites(state: State<AppState>) -> CommandResult<Vec<VaultItemDto>> {
    state.touch();
    let vault = state.vault.lock().unwrap();
    let vault = vault.as_ref().ok_or(CommandError {
        message: "Vault is locked".to_string(),
    })?;

    Ok(vault
        .get_favorites()
        .iter()
        .map(|i| (*i).into())
        .collect())
}

// =============================================================================
// Password Generation Commands
// =============================================================================

#[derive(Deserialize)]
pub struct PasswordOptionsDto {
    pub length: Option<usize>,
    pub lowercase: Option<bool>,
    pub uppercase: Option<bool>,
    pub digits: Option<bool>,
    pub symbols: Option<bool>,
    pub exclude_ambiguous: Option<bool>,
    pub exclude_chars: Option<String>,
}

#[tauri::command]
pub fn generate_password_cmd(options: PasswordOptionsDto) -> CommandResult<String> {
    let opts = PasswordOptions {
        length: options.length.unwrap_or(16),
        lowercase: options.lowercase.unwrap_or(true),
        uppercase: options.uppercase.unwrap_or(true),
        digits: options.digits.unwrap_or(true),
        symbols: options.symbols.unwrap_or(true),
        exclude_ambiguous: options.exclude_ambiguous.unwrap_or(false),
        exclude_chars: options.exclude_chars.unwrap_or_default(),
    };

    generate_password(&opts).map_err(|e| e.into())
}

#[tauri::command]
pub fn generate_passphrase_cmd(word_count: usize, separator: String) -> CommandResult<String> {
    generate_passphrase(word_count, &separator).map_err(|e| e.into())
}

// =============================================================================
// Settings Commands
// =============================================================================

#[tauri::command]
pub fn get_auto_lock_timeout(state: State<AppState>) -> CommandResult<u64> {
    Ok(*state.auto_lock_timeout.lock().unwrap())
}

#[tauri::command]
pub fn set_auto_lock_timeout(timeout: u64, state: State<AppState>) -> CommandResult<()> {
    *state.auto_lock_timeout.lock().unwrap() = timeout;
    let storage = Storage::open()?;
    storage.set_setting("auto_lock_timeout", &timeout.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn check_auto_lock(state: State<AppState>) -> CommandResult<bool> {
    if state.is_unlocked() && state.should_auto_lock() {
        state.lock();
        return Ok(true);
    }
    Ok(false)
}

// =============================================================================
// Sync Commands
// =============================================================================

#[tauri::command]
pub fn get_sync_status(sync_state: State<SyncState>) -> CommandResult<SyncStatus> {
    Ok(sync_state.get_status())
}

#[derive(Deserialize)]
pub struct EnableSyncRequest {
    pub server_url: String,
    pub access_token: String,
    pub device_id: String,
}

#[tauri::command]
pub fn enable_sync(request: EnableSyncRequest, sync_state: State<SyncState>) -> CommandResult<()> {
    sync_state.enable(request.server_url, request.access_token, request.device_id);
    Ok(())
}

#[tauri::command]
pub fn disable_sync(sync_state: State<SyncState>) -> CommandResult<()> {
    sync_state.disable();
    Ok(())
}

#[tauri::command]
pub fn trigger_sync(sync_state: State<SyncState>) -> CommandResult<()> {
    if !sync_state.is_enabled() {
        return Err(CommandError {
            message: "Sync is not enabled".to_string(),
        });
    }

    // Set syncing state
    sync_state.set_syncing();

    // In a full implementation, this would:
    // 1. Pull changes from server
    // 2. Push local changes
    // 3. Update sync status

    // For now, simulate completion
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    sync_state.set_idle(now);

    Ok(())
}

#[tauri::command]
pub fn check_remote_commands(sync_state: State<SyncState>) -> CommandResult<Vec<RemoteCommand>> {
    if !sync_state.is_enabled() {
        return Ok(vec![]);
    }

    // In a full implementation, this would:
    // 1. Call the API to get pending commands
    // 2. Return them for the frontend to handle

    // For now, return empty
    Ok(vec![])
}

// =============================================================================
// Wipe Vault Command
// =============================================================================

#[tauri::command]
pub fn wipe_vault(app_state: State<AppState>, sync_state: State<SyncState>) -> CommandResult<()> {
    // Lock the vault first
    app_state.lock();

    // Disable sync
    sync_state.disable();

    // Delete the vault file
    let storage = Storage::open()?;
    storage.delete_vault()?;

    Ok(())
}
