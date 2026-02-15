// Storage utilities for the extension

export interface VaultItem {
  id: string;
  name: string;
  url: string | null;
  username: string;
  password: string;
  notes: string | null;
  category: string | null;
  favorite: boolean;
  createdAt: number;
  modifiedAt: number;
}

export interface VaultData {
  version: number;
  items: VaultItem[];
  categories: string[];
  lastSync: number | null;
}

export interface StoredData {
  salt: string; // base64
  encryptedVault: string; // base64 JSON of EncryptedBlob
  settings: {
    autoLockTimeout: number;
    autoFillEnabled: boolean;
    showNotifications: boolean;
  };
}

const STORAGE_KEY = 'keydrop_vault';

// Check if vault exists
export async function vaultExists(): Promise<boolean> {
  const result = await chrome.storage.local.get(STORAGE_KEY);
  const data = result[STORAGE_KEY] as StoredData | undefined;
  return !!data?.salt;
}

// Get stored data
export async function getStoredData(): Promise<StoredData | null> {
  const result = await chrome.storage.local.get(STORAGE_KEY);
  return (result[STORAGE_KEY] as StoredData | undefined) || null;
}

// Save stored data
export async function saveStoredData(data: StoredData): Promise<void> {
  await chrome.storage.local.set({ [STORAGE_KEY]: data });
}

// Get salt
export async function getSalt(): Promise<string | null> {
  const data = await getStoredData();
  return data?.salt || null;
}

// Save encrypted vault
export async function saveEncryptedVault(
  salt: string,
  encryptedVault: string
): Promise<void> {
  const existing = await getStoredData();
  await saveStoredData({
    salt,
    encryptedVault,
    settings: existing?.settings || {
      autoLockTimeout: 300,
      autoFillEnabled: true,
      showNotifications: true,
    },
  });
}

// Get encrypted vault
export async function getEncryptedVault(): Promise<string | null> {
  const data = await getStoredData();
  return data?.encryptedVault || null;
}

// Get settings
export async function getSettings(): Promise<StoredData['settings']> {
  const data = await getStoredData();
  return (
    data?.settings || {
      autoLockTimeout: 300,
      autoFillEnabled: true,
      showNotifications: true,
    }
  );
}

// Update settings
export async function updateSettings(
  settings: Partial<StoredData['settings']>
): Promise<void> {
  const data = await getStoredData();
  if (data) {
    data.settings = { ...data.settings, ...settings };
    await saveStoredData(data);
  }
}

// Create empty vault
export function createEmptyVault(): VaultData {
  return {
    version: 1,
    items: [],
    categories: ['Login', 'Credit Card', 'Identity', 'Secure Note'],
    lastSync: null,
  };
}

// Generate UUID
export function generateId(): string {
  return crypto.randomUUID();
}
