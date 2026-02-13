// Crypto utilities for the extension
// In production, this would use the WASM crypto-core module
// For MVP, we use Web Crypto API with compatible algorithms

const encoder = new TextEncoder();
const decoder = new TextDecoder();

export interface EncryptedBlob {
  nonce: string; // base64
  ciphertext: string; // base64
}

export interface KeySet {
  vaultKey: CryptoKey;
  authKey: CryptoKey;
}

// Generate a random salt
export async function generateSalt(): Promise<Uint8Array> {
  return crypto.getRandomValues(new Uint8Array(16));
}

// Derive master key from password using PBKDF2 (Web Crypto compatible)
// Note: In production, use Argon2id via WASM for better security
export async function deriveMasterKey(
  password: string,
  salt: Uint8Array
): Promise<CryptoKey> {
  const keyMaterial = await crypto.subtle.importKey(
    'raw',
    encoder.encode(password),
    'PBKDF2',
    false,
    ['deriveBits', 'deriveKey']
  );

  return crypto.subtle.deriveKey(
    {
      name: 'PBKDF2',
      salt: salt as Uint8Array<ArrayBuffer>,
      iterations: 600000, // OWASP recommended
      hash: 'SHA-256',
    },
    keyMaterial,
    { name: 'AES-GCM', length: 256 },
    true,
    ['encrypt', 'decrypt']
  );
}

// Derive vault and auth keys from master key
export async function deriveKeys(masterKey: CryptoKey): Promise<KeySet> {
  const masterKeyBytes = await crypto.subtle.exportKey('raw', masterKey);

  // Derive vault key
  const vaultKeyMaterial = await crypto.subtle.importKey(
    'raw',
    masterKeyBytes,
    'HKDF',
    false,
    ['deriveKey']
  );

  const vaultKey = await crypto.subtle.deriveKey(
    {
      name: 'HKDF',
      hash: 'SHA-256',
      salt: new Uint8Array(0),
      info: encoder.encode('keydrop-vault-key'),
    },
    vaultKeyMaterial,
    { name: 'AES-GCM', length: 256 },
    false,
    ['encrypt', 'decrypt']
  );

  // Derive auth key
  const authKeyMaterial = await crypto.subtle.importKey(
    'raw',
    masterKeyBytes,
    'HKDF',
    false,
    ['deriveKey']
  );

  const authKey = await crypto.subtle.deriveKey(
    {
      name: 'HKDF',
      hash: 'SHA-256',
      salt: new Uint8Array(0),
      info: encoder.encode('keydrop-auth-key'),
    },
    authKeyMaterial,
    { name: 'AES-GCM', length: 256 },
    false,
    ['encrypt', 'decrypt']
  );

  return { vaultKey, authKey };
}

// Encrypt data using AES-256-GCM
export async function encrypt(
  data: string,
  key: CryptoKey
): Promise<EncryptedBlob> {
  const nonce = crypto.getRandomValues(new Uint8Array(12));
  const ciphertext = await crypto.subtle.encrypt(
    { name: 'AES-GCM', iv: nonce },
    key,
    encoder.encode(data)
  );

  return {
    nonce: btoa(String.fromCharCode(...nonce)),
    ciphertext: btoa(String.fromCharCode(...new Uint8Array(ciphertext))),
  };
}

// Decrypt data using AES-256-GCM
export async function decrypt(
  blob: EncryptedBlob,
  key: CryptoKey
): Promise<string> {
  const nonce = Uint8Array.from(atob(blob.nonce), (c) => c.charCodeAt(0));
  const ciphertext = Uint8Array.from(atob(blob.ciphertext), (c) =>
    c.charCodeAt(0)
  );

  const plaintext = await crypto.subtle.decrypt(
    { name: 'AES-GCM', iv: nonce },
    key,
    ciphertext
  );

  return decoder.decode(plaintext);
}

// Generate a random password
export function generatePassword(options: {
  length: number;
  lowercase: boolean;
  uppercase: boolean;
  digits: boolean;
  symbols: boolean;
}): string {
  let charset = '';
  const required: string[] = [];

  if (options.lowercase) {
    charset += 'abcdefghijklmnopqrstuvwxyz';
    required.push('abcdefghijklmnopqrstuvwxyz'[Math.floor(Math.random() * 26)]);
  }
  if (options.uppercase) {
    charset += 'ABCDEFGHIJKLMNOPQRSTUVWXYZ';
    required.push('ABCDEFGHIJKLMNOPQRSTUVWXYZ'[Math.floor(Math.random() * 26)]);
  }
  if (options.digits) {
    charset += '0123456789';
    required.push('0123456789'[Math.floor(Math.random() * 10)]);
  }
  if (options.symbols) {
    charset += '!@#$%^&*()_+-=[]{}|;:,.<>?';
    required.push(
      '!@#$%^&*()_+-=[]{}|;:,.<>?'[Math.floor(Math.random() * 26)]
    );
  }

  if (!charset) {
    throw new Error('At least one character type must be enabled');
  }

  const randomBytes = crypto.getRandomValues(new Uint8Array(options.length));
  let password = Array.from(randomBytes)
    .map((byte) => charset[byte % charset.length])
    .join('');

  // Ensure at least one character from each required set
  const passwordArray = password.split('');
  required.forEach((char, index) => {
    if (index < passwordArray.length) {
      passwordArray[index] = char;
    }
  });

  // Shuffle
  for (let i = passwordArray.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    [passwordArray[i], passwordArray[j]] = [passwordArray[j], passwordArray[i]];
  }

  return passwordArray.join('');
}

// Utility to convert ArrayBuffer to base64
export function arrayBufferToBase64(buffer: ArrayBuffer): string {
  return btoa(String.fromCharCode(...new Uint8Array(buffer)));
}

// Utility to convert base64 to Uint8Array
export function base64ToUint8Array(base64: string): Uint8Array {
  return Uint8Array.from(atob(base64), (c) => c.charCodeAt(0));
}
