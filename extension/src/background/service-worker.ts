// Background service worker for Keydrop extension

import {
  deriveMasterKey,
  deriveKeys,
  encrypt,
  decrypt,
  generatePassword,
  generateSalt,
  base64ToUint8Array,
  arrayBufferToBase64,
  type KeySet,
  type EncryptedBlob,
} from '../lib/crypto';
import {
  vaultExists,
  getSalt,
  saveEncryptedVault,
  getEncryptedVault,
  createEmptyVault,
  generateId,
  type VaultData,
  type VaultItem,
} from '../lib/storage';

// In-memory state (cleared on service worker restart)
let unlockedVault: VaultData | null = null;
let keys: KeySet | null = null;
let lastActivity = 0;
let autoLockTimeout = 300; // 5 minutes

// Message types
type MessageType =
  | { type: 'GET_STATUS' }
  | { type: 'CREATE_VAULT'; password: string }
  | { type: 'UNLOCK'; password: string }
  | { type: 'LOCK' }
  | { type: 'GET_ITEMS' }
  | { type: 'GET_ITEMS_FOR_URL'; url: string }
  | { type: 'ADD_ITEM'; item: Omit<VaultItem, 'id' | 'createdAt' | 'modifiedAt'> }
  | { type: 'UPDATE_ITEM'; item: VaultItem }
  | { type: 'DELETE_ITEM'; id: string }
  | { type: 'GENERATE_PASSWORD'; options: { length: number; lowercase: boolean; uppercase: boolean; digits: boolean; symbols: boolean } }
  | { type: 'AUTOFILL'; tabId: number; item: VaultItem };

interface MessageResponse {
  success: boolean;
  data?: unknown;
  error?: string;
}

// Touch activity timestamp
function touch() {
  lastActivity = Date.now();
}

// Check if should auto-lock
function shouldAutoLock(): boolean {
  if (!unlockedVault) return false;
  return Date.now() - lastActivity > autoLockTimeout * 1000;
}

// Lock vault
function lock() {
  unlockedVault = null;
  keys = null;
}

// Save vault to storage
async function saveVault(): Promise<void> {
  if (!unlockedVault || !keys) return;

  const json = JSON.stringify(unlockedVault);
  const encrypted = await encrypt(json, keys.vaultKey);
  const salt = await getSalt();

  if (salt) {
    await saveEncryptedVault(salt, JSON.stringify(encrypted));
  }
}

// Extract domain from URL
function extractDomain(url: string): string {
  try {
    const parsed = new URL(url);
    return parsed.hostname.replace(/^www\./, '');
  } catch {
    return url;
  }
}

// Check if domains match
function domainsMatch(domain1: string, domain2: string): boolean {
  if (domain1 === domain2) return true;
  return (
    domain1.endsWith(`.${domain2}`) || domain2.endsWith(`.${domain1}`)
  );
}

// Handle messages from popup and content scripts
chrome.runtime.onMessage.addListener(
  (
    message: MessageType,
    _sender,
    sendResponse: (response: MessageResponse) => void
  ) => {
    handleMessage(message).then(sendResponse);
    return true; // Will respond asynchronously
  }
);

async function handleMessage(message: MessageType): Promise<MessageResponse> {
  // Check auto-lock
  if (shouldAutoLock()) {
    lock();
  }

  try {
    switch (message.type) {
      case 'GET_STATUS': {
        const exists = await vaultExists();
        return {
          success: true,
          data: {
            vaultExists: exists,
            unlocked: !!unlockedVault,
            itemCount: unlockedVault?.items.length || 0,
          },
        };
      }

      case 'CREATE_VAULT': {
        if (await vaultExists()) {
          return { success: false, error: 'Vault already exists' };
        }

        const salt = await generateSalt();
        const masterKey = await deriveMasterKey(message.password, salt);
        keys = await deriveKeys(masterKey);
        unlockedVault = createEmptyVault();

        const json = JSON.stringify(unlockedVault);
        const encrypted = await encrypt(json, keys.vaultKey);
        await saveEncryptedVault(
          arrayBufferToBase64(salt.buffer),
          JSON.stringify(encrypted)
        );

        touch();
        return { success: true };
      }

      case 'UNLOCK': {
        const saltBase64 = await getSalt();
        if (!saltBase64) {
          return { success: false, error: 'No vault found' };
        }

        const salt = base64ToUint8Array(saltBase64);
        const masterKey = await deriveMasterKey(message.password, salt);
        keys = await deriveKeys(masterKey);

        const encryptedJson = await getEncryptedVault();
        if (!encryptedJson) {
          return { success: false, error: 'No vault data found' };
        }

        try {
          const encrypted: EncryptedBlob = JSON.parse(encryptedJson);
          const json = await decrypt(encrypted, keys.vaultKey);
          unlockedVault = JSON.parse(json);
          touch();
          return { success: true };
        } catch {
          keys = null;
          return { success: false, error: 'Invalid password' };
        }
      }

      case 'LOCK': {
        lock();
        return { success: true };
      }

      case 'GET_ITEMS': {
        touch();
        if (!unlockedVault) {
          return { success: false, error: 'Vault is locked' };
        }
        return { success: true, data: unlockedVault.items };
      }

      case 'GET_ITEMS_FOR_URL': {
        touch();
        if (!unlockedVault) {
          return { success: false, error: 'Vault is locked' };
        }

        const domain = extractDomain(message.url);
        const matches = unlockedVault.items.filter((item) => {
          if (!item.url) return false;
          const itemDomain = extractDomain(item.url);
          return domainsMatch(domain, itemDomain);
        });

        return { success: true, data: matches };
      }

      case 'ADD_ITEM': {
        touch();
        if (!unlockedVault) {
          return { success: false, error: 'Vault is locked' };
        }

        const now = Date.now();
        const newItem: VaultItem = {
          ...message.item,
          id: generateId(),
          createdAt: now,
          modifiedAt: now,
        };

        unlockedVault.items.push(newItem);
        await saveVault();

        return { success: true, data: newItem.id };
      }

      case 'UPDATE_ITEM': {
        touch();
        if (!unlockedVault) {
          return { success: false, error: 'Vault is locked' };
        }

        const index = unlockedVault.items.findIndex(
          (i) => i.id === message.item.id
        );
        if (index === -1) {
          return { success: false, error: 'Item not found' };
        }

        unlockedVault.items[index] = {
          ...message.item,
          modifiedAt: Date.now(),
        };
        await saveVault();

        return { success: true };
      }

      case 'DELETE_ITEM': {
        touch();
        if (!unlockedVault) {
          return { success: false, error: 'Vault is locked' };
        }

        const itemIndex = unlockedVault.items.findIndex(
          (i) => i.id === message.id
        );
        if (itemIndex === -1) {
          return { success: false, error: 'Item not found' };
        }

        unlockedVault.items.splice(itemIndex, 1);
        await saveVault();

        return { success: true };
      }

      case 'GENERATE_PASSWORD': {
        const password = generatePassword(message.options);
        return { success: true, data: password };
      }

      case 'AUTOFILL': {
        touch();
        // Send autofill command to content script
        chrome.tabs.sendMessage(message.tabId, {
          type: 'AUTOFILL',
          username: message.item.username,
          password: message.item.password,
        });
        return { success: true };
      }

      default:
        return { success: false, error: 'Unknown message type' };
    }
  } catch (error) {
    return {
      success: false,
      error: error instanceof Error ? error.message : 'Unknown error',
    };
  }
}

// Auto-lock check interval
setInterval(() => {
  if (shouldAutoLock()) {
    lock();
  }
}, 10000);

// Log service worker startup
console.log('Keydrop service worker started');
