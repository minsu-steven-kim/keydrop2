import { invoke } from '@tauri-apps/api/tauri';

export interface VaultStatus {
  exists: boolean;
  unlocked: boolean;
}

export interface VaultItem {
  id: string;
  name: string;
  url: string | null;
  username: string;
  password: string;
  notes: string | null;
  category: string | null;
  favorite: boolean;
  created_at: number;
  modified_at: number;
}

export interface PasswordOptions {
  length?: number;
  lowercase?: boolean;
  uppercase?: boolean;
  digits?: boolean;
  symbols?: boolean;
  exclude_ambiguous?: boolean;
  exclude_chars?: string;
}

export const tauri = {
  // Vault status
  getVaultStatus: () => invoke<VaultStatus>('get_vault_status'),

  // Vault operations
  createVault: (password: string) => invoke<void>('create_vault', { password }),
  unlockVault: (password: string) => invoke<void>('unlock_vault', { password }),
  lockVault: () => invoke<void>('lock_vault'),

  // Item operations
  getAllItems: () => invoke<VaultItem[]>('get_all_items'),
  getItem: (id: string) => invoke<VaultItem | null>('get_item', { id }),
  addItem: (item: VaultItem) => invoke<string>('add_item', { item }),
  updateItem: (id: string, item: VaultItem) => invoke<void>('update_item', { id, item }),
  deleteItem: (id: string) => invoke<void>('delete_item', { id }),
  searchItems: (query: string) => invoke<VaultItem[]>('search_items', { query }),
  getFavorites: () => invoke<VaultItem[]>('get_favorites'),

  // Password generation
  generatePassword: (options: PasswordOptions) =>
    invoke<string>('generate_password_cmd', { options }),
  generatePassphrase: (wordCount: number, separator: string) =>
    invoke<string>('generate_passphrase_cmd', { wordCount, separator }),

  // Settings
  getAutoLockTimeout: () => invoke<number>('get_auto_lock_timeout'),
  setAutoLockTimeout: (timeout: number) =>
    invoke<void>('set_auto_lock_timeout', { timeout }),
  checkAutoLock: () => invoke<boolean>('check_auto_lock'),
};
