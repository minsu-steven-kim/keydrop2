# AGENTS.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Keydrop is a universal password manager with three client platforms (Chrome extension, desktop app, Android app) sharing a unified encrypted vault via a zero-knowledge sync service.

## Architecture

### Multi-Platform Structure

```
keydrop/
├── crypto-core/     # Rust shared crypto library (WASM + JNI bindings)
├── extension/       # Chrome extension (Manifest V3, TypeScript, React)
├── desktop/         # Desktop app (Tauri, React)
├── android/         # Android app (Kotlin, Jetpack Compose)
├── backend/         # Sync service (Go or Rust, PostgreSQL)
└── shared/          # Shared TypeScript types and utilities
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
