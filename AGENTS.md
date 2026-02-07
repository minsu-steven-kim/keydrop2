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
│   ├── uniffi/            # UniFFI bindings for Android/iOS
│   └── benches/           # Performance benchmarks
├── desktop/               # Desktop app (Tauri + React)
│   ├── src/               # React frontend (components, hooks, lib)
│   └── src-tauri/         # Rust backend (commands, storage, state)
├── extension/             # Chrome extension (Manifest V3)
│   ├── src/background/    # Service worker
│   ├── src/content/       # Content scripts (detector, autofill)
│   ├── src/popup/         # Popup UI (React)
│   └── src/lib/           # Shared utilities
├── android/               # Android app (Kotlin, Jetpack Compose)
│   └── app/               # Main application module
└── backend/               # Sync backend (Rust + Axum, PostgreSQL)
    ├── src/               # API, auth, sync, blob modules
    └── migrations/        # PostgreSQL schema migrations
```

### Crypto Core

All clients use the same Rust crypto library with platform-specific bindings:
- **WASM** for Chrome extension and desktop frontend
- **UniFFI** for Android (generates Kotlin bindings)
- **Native** for Tauri backend and sync server

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

# UniFFI bindings (for Android)
cd crypto-core/uniffi && cargo build --release

# Desktop app
cd desktop && npm install && npm run tauri dev

# Chrome extension
cd extension && npm install && npm run build

# Sync backend
cd backend && cargo run

# Android app
cd android && ./gradlew assembleDebug
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

### Backend
- `backend/src/main.rs` - Server entry point, Axum router setup
- `backend/src/lib.rs` - Library exports, AppState definition
- `backend/src/api/auth.rs` - Register, login, refresh token endpoints
- `backend/src/api/sync.rs` - Pull/push sync endpoints, WebSocket handler
- `backend/src/api/devices.rs` - Device management, biometric auth requests
- `backend/src/auth/jwt.rs` - JWT generation and validation
- `backend/src/db/models.rs` - SQLx database models
- `backend/src/blob/mod.rs` - S3-compatible blob storage
- `backend/src/sync/conflict.rs` - Last-write-wins conflict resolution
- `backend/migrations/` - PostgreSQL schema migrations

### Android
- `android/app/build.gradle.kts` - App configuration with Compose, Hilt, Room
- `android/app/src/main/java/com/keydrop/KeydropApplication.kt` - Hilt application
- `android/app/src/main/java/com/keydrop/ui/` - Jetpack Compose screens
- `android/app/src/main/java/com/keydrop/data/` - Room database, repositories
- `android/app/src/main/java/com/keydrop/sync/` - SyncManager, SyncWorker
- `android/app/src/main/java/com/keydrop/biometric/` - BiometricManager
- `android/app/src/main/java/com/keydrop/autofill/` - Android Autofill Service
- `android/app/src/main/java/com/keydrop/widget/` - Quick access widget

### UniFFI Bindings
- `crypto-core/uniffi/src/lib.rs` - UniFFI bindings wrapping crypto-core
- `crypto-core/uniffi/src/crypto_core.udl` - UniFFI interface definition
