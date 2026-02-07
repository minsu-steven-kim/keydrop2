# Keydrop

[![CI](https://github.com/minsu-steven-kim/keydrop2/actions/workflows/ci.yml/badge.svg)](https://github.com/minsu-steven-kim/keydrop2/actions/workflows/ci.yml)

A secure, cross-platform password manager with zero-knowledge architecture.

## Overview

Keydrop is a universal password manager that provides encrypted credential storage across multiple platforms. All cryptographic operations happen client-side, ensuring the server never has access to plaintext data.

### Platforms

- **Desktop App** - Full-featured vault management (Windows, macOS, Linux)
- **Chrome Extension** - Auto-fill and quick access in the browser
- **Android App** - Mobile access with biometric authentication
- **Sync Backend** - Zero-knowledge sync server (Rust + PostgreSQL)

## Project Structure

```
keydrop/
├── crypto-core/          # Rust cryptographic library
│   ├── src/              # Core crypto implementation
│   ├── wasm/             # WebAssembly bindings
│   └── uniffi/           # UniFFI bindings for Android/iOS
├── desktop/              # Tauri desktop application
│   ├── src/              # React frontend
│   └── src-tauri/        # Rust backend
├── extension/            # Chrome extension (Manifest V3)
│   ├── src/background/   # Service worker
│   ├── src/content/      # Content scripts
│   └── src/popup/        # Popup UI
├── android/              # Android app (Kotlin + Jetpack Compose)
│   └── app/              # Main application module
├── backend/              # Sync backend (Rust + Axum)
│   ├── src/              # API, auth, sync, blob modules
│   └── migrations/       # PostgreSQL migrations
└── docs/
    ├── architecture.md   # System architecture
    └── requirements.md   # Functional requirements
```

## Security

Keydrop uses a zero-knowledge architecture:

- **Encryption**: AES-256-GCM authenticated encryption
- **Key Derivation**: Argon2id for master password, HKDF for key hierarchy
- **Memory Protection**: Sensitive data zeroized after use
- **No Plaintext on Server**: All encryption/decryption happens client-side

### Key Hierarchy

```
Master Password
       │
       ▼ (Argon2id)
Master Key
       │
       ├──▶ Vault Key (encrypts credentials)
       ├──▶ Auth Key (server authentication)
       └──▶ Sharing Key (secure sharing)
```

## Getting Started

### Prerequisites

- Rust 1.70+ with `wasm32-unknown-unknown` target
- Node.js 18+
- pnpm or npm

### Building

#### Crypto Core

```bash
cd crypto-core
cargo build --release

# Build WASM bindings
cd wasm
wasm-pack build --target web
```

#### Desktop App

```bash
cd desktop
npm install
npm run tauri dev
```

#### Chrome Extension

```bash
cd extension
npm install
npm run build
# Load dist/ folder as unpacked extension in Chrome
```

#### Sync Backend

```bash
cd backend
# Requires PostgreSQL and S3-compatible storage
export DATABASE_URL=postgres://keydrop:keydrop@localhost/keydrop
cargo run
```

#### Android App

```bash
cd android
# Build native crypto libraries first
cd ../crypto-core/uniffi
cargo ndk -t arm64-v8a -o ../../android/app/src/main/jniLibs build --release

# Build app
cd ../../android
./gradlew assembleDebug
```

## Development

### Running Tests

```bash
# Crypto core tests
cd crypto-core
cargo test

# Benchmarks
cargo bench
```

### Tech Stack

| Component | Technology |
|-----------|------------|
| Crypto Core | Rust, argon2, aes-gcm, hkdf |
| Desktop | Tauri, React, TypeScript |
| Extension | Chrome Manifest V3, TypeScript, React |
| Android | Kotlin, Jetpack Compose, Hilt, Room |
| Backend | Rust, Axum, SQLx, PostgreSQL |
| Blob Storage | S3-compatible (AWS S3, MinIO, R2) |
| Local Storage | SQLCipher (encrypted SQLite) |

## CI/CD

This project uses GitHub Actions for continuous integration and deployment.

### Workflows

- **CI** (`ci.yml`) - Runs on every push/PR: tests, builds all platforms
- **Release** (`release.yml`) - Triggered by version tags: builds releases, deploys backend

### Required Secrets

| Secret | Description |
|--------|-------------|
| `TAURI_SIGNING_PRIVATE_KEY` | Tauri app signing key |
| `ANDROID_KEYSTORE` | Base64-encoded Android keystore |
| `ANDROID_KEYSTORE_PASSWORD` | Keystore password |
| `ANDROID_KEY_ALIAS` | Key alias in keystore |
| `ANDROID_KEY_PASSWORD` | Key password |
| `FLY_API_TOKEN` | Fly.io API token for backend deployment |

## Documentation

- [Architecture](architecture.md) - System design and component details
- [Requirements](requirements.md) - Functional and non-functional requirements
- [Deployment](DEPLOYMENT.md) - Production deployment guide
- [AGENTS.md](AGENTS.md) - Guidelines for AI coding assistants

## License

MIT
