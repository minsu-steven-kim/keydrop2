# AGENTS.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Keydrop is a universal password manager with three client platforms (Chrome extension, desktop app, Android app) sharing a unified encrypted vault via a zero-knowledge sync service.

## Architecture

### Multi-Platform Structure

```
keydrop/
├── crypto-core/           # Rust shared crypto library
│   ├── src/               # Core implementation (cipher, kdf, password, vault)
│   ├── wasm/              # WebAssembly bindings for browser/desktop frontend
│   └── benches/           # Performance benchmarks
├── desktop/               # Desktop app (Tauri + React)
│   ├── src/               # React frontend (components, hooks, lib)
│   └── src-tauri/         # Rust backend (commands, storage, state)
├── extension/             # Chrome extension (Manifest V3)
│   ├── src/background/    # Service worker
│   ├── src/content/       # Content scripts (detector, autofill)
│   ├── src/popup/         # Popup UI (React)
│   └── src/lib/           # Shared utilities
├── android/               # Android app (planned - Kotlin, Jetpack Compose)
└── backend/               # Sync service (planned - Go or Rust, PostgreSQL)
```

### Crypto Core

All clients use the same Rust crypto library with platform-specific bindings:
- **WASM** for Chrome extension and desktop frontend
- **JNI** for Android
- **Native** for Tauri backend

Key algorithms: Argon2id (key derivation), AES-256-GCM (encryption), X25519 (key exchange), HKDF (key hierarchy).

### Key Hierarchy

Master Password → (Argon2id) → Master Key → (HKDF) → {Vault Key, Auth Key, Sharing Key}

The master password never leaves the client. Server authentication uses a separate derived auth key.

### Biometric Authentication

Android acts as the biometric authenticator for all clients. The encrypted master key is stored in Android Keystore and released to requesting clients (Chrome/desktop) via secure local channel after biometric success.

### Sync Protocol

Delta-based sync with versioning. Server stores only encrypted blobs (zero-knowledge). Conflict resolution: last-write-wins with field-level merge.

## Security Requirements

- All vault data encrypted with AES-256-GCM before leaving the client
- Zero-knowledge: server never sees plaintext credentials
- Memory protection: zeroize sensitive data after use
- Certificate pinning for API communication
- Use only audited crypto libraries (ring, RustCrypto)

## Platform Constraints

- Chrome extension: Manifest V3 (service workers, no persistent background)
- Android: minimum SDK 26 (Android 8.0)
- Desktop: Windows 10+, macOS 11+, Ubuntu 20.04+

## Build Commands

```bash
# Crypto core
cd crypto-core && cargo build --release
cd crypto-core && cargo test
cd crypto-core && cargo bench

# WASM bindings
cd crypto-core/wasm && wasm-pack build --target web

# Desktop app
cd desktop && npm install && npm run tauri dev

# Chrome extension
cd extension && npm install && npm run build
```

## Key Files

### Crypto Core
- `crypto-core/src/lib.rs` - Public API and module exports
- `crypto-core/src/kdf.rs` - Argon2id key derivation, HKDF key expansion
- `crypto-core/src/cipher.rs` - AES-256-GCM encryption/decryption
- `crypto-core/src/vault.rs` - Vault and VaultItem types, search, import/export
- `crypto-core/src/password.rs` - Password and passphrase generation
- `crypto-core/wasm/src/lib.rs` - WASM bindings via wasm-bindgen

### Desktop
- `desktop/src-tauri/src/commands.rs` - Tauri command handlers
- `desktop/src-tauri/src/storage.rs` - Local vault persistence
- `desktop/src/components/UnlockScreen.tsx` - Master password entry
- `desktop/src/components/VaultList.tsx` - Credential list display
- `desktop/src/components/CredentialForm.tsx` - Add/edit credentials
- `desktop/src/components/PasswordGenerator.tsx` - Password generation UI
- `desktop/src/lib/crypto.ts` - Frontend crypto wrapper

### Extension
- `extension/manifest.json` - Extension manifest (MV3)
- `extension/src/background/service-worker.ts` - Background service worker
- `extension/src/content/detector.ts` - Login form detection
- `extension/src/content/autofill.ts` - Auto-fill injection
- `extension/src/popup/Popup.tsx` - Quick access popup
- `extension/src/lib/crypto.ts` - WASM crypto wrapper
- `extension/src/lib/storage.ts` - Chrome storage abstraction
