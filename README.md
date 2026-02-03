# Keydrop

A secure, cross-platform password manager with zero-knowledge architecture.

## Overview

Keydrop is a universal password manager that provides encrypted credential storage across multiple platforms. All cryptographic operations happen client-side, ensuring the server never has access to plaintext data.

### Platforms

- **Desktop App** - Full-featured vault management (Windows, macOS, Linux)
- **Chrome Extension** - Auto-fill and quick access in the browser
- **Android App** - Mobile access with biometric authentication (planned)

## Project Structure

```
keydrop/
├── crypto-core/          # Rust cryptographic library
│   ├── src/              # Core crypto implementation
│   └── wasm/             # WebAssembly bindings
├── desktop/              # Tauri desktop application
│   ├── src/              # React frontend
│   └── src-tauri/        # Rust backend
├── extension/            # Chrome extension (Manifest V3)
│   ├── src/background/   # Service worker
│   ├── src/content/      # Content scripts
│   └── src/popup/        # Popup UI
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
| Local Storage | SQLCipher (encrypted SQLite) |

## Documentation

- [Architecture](architecture.md) - System design and component details
- [Requirements](requirements.md) - Functional and non-functional requirements
- [AGENTS.md](AGENTS.md) - Guidelines for AI coding assistants

## License

MIT
