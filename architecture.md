# Keydrop - Architecture

## System Overview

```
┌─────────────────────────────────────────────────────────────────────────┐
│                              Clients                                     │
├──────────────────┬──────────────────────┬───────────────────────────────┤
│  Chrome Extension│    Desktop App       │       Android App             │
│  (Manifest V3)   │    (Tauri + React)   │       (Kotlin) [Planned]      │
│                  │                      │                               │
│  - Auto-fill     │  - Vault management  │  - Biometric auth provider    │
│  - Quick access  │  - Import/Export     │  - Autofill service           │
│  - Form detection│  - Health reports    │  - Mobile access              │
└────────┬─────────┴──────────┬───────────┴───────────────┬───────────────┘
         │                    │                           │
         └────────────────────┼───────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                    Sync Service (Cloud) [Planned]                        │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────────────┐  │
│  │   Auth Service  │  │   Sync Engine   │  │   Encrypted Blob Store  │  │
│  │                 │  │                 │  │                         │  │
│  │  - JWT tokens   │  │  - Versioning   │  │  - User vaults          │  │
│  │  - Device reg   │  │  - Conflicts    │  │  - Zero-knowledge       │  │
│  │  - Rate limit   │  │  - Push notify  │  │  - Backup retention     │  │
│  └─────────────────┘  └─────────────────┘  └─────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────┘
```

> **Implementation Status**: Chrome Extension and Desktop App are implemented.
> Android App and Sync Service are planned for future development.

## Core Components

### 1. Crypto Core (Shared Library)

Cross-platform cryptographic library used by all clients.

```
┌─────────────────────────────────────────┐
│             Crypto Core                 │
├─────────────────────────────────────────┤
│  Key Derivation (kdf.rs)                │
│  - Argon2id (master password → key)     │
│  - HKDF-SHA256 (key → vault/auth keys)  │
├─────────────────────────────────────────┤
│  Encryption (cipher.rs)                 │
│  - AES-256-GCM (vault items)            │
│  - Random nonce generation              │
├─────────────────────────────────────────┤
│  Vault Management (vault.rs)            │
│  - VaultItem CRUD operations            │
│  - Search and URL matching              │
│  - Encrypted import/export              │
├─────────────────────────────────────────┤
│  Password Generation (password.rs)      │
│  - Configurable length/complexity       │
│  - Passphrase generation                │
└─────────────────────────────────────────┘
```

**Implementation**: Rust library using RustCrypto crates (argon2, aes-gcm, hkdf, sha2).
- **Native**: Used directly by Tauri desktop backend
- **WASM**: Compiled via wasm-pack for browser extension and desktop frontend
- **JNI**: Planned for Android (not yet implemented)

### 2. Vault Structure

```
Vault
├── metadata (encrypted)
│   ├── version
│   ├── created_at
│   └── last_modified
├── folders[]
│   ├── id
│   ├── name
│   └── parent_id
└── items[]
    ├── id
    ├── type (login | card | note | identity)
    ├── folder_id
    ├── name
    ├── encrypted_data
    │   └── (type-specific fields)
    ├── created_at
    ├── modified_at
    └── history[]
```

### 3. Authentication Flow

#### Master Password Authentication

```
┌──────────┐     ┌─────────────┐     ┌─────────────┐     ┌───────────┐
│  User    │────▶│  Enter      │────▶│  Derive     │────▶│  Decrypt  │
│  Input   │     │  Master PW  │     │  Keys       │     │  Vault    │
└──────────┘     └─────────────┘     └─────────────┘     └───────────┘
                                            │
                                            ▼
                                     ┌─────────────┐
                                     │  Argon2id   │
                                     │  (100ms+)   │
                                     └─────────────┘
```

#### Biometric Authentication (Android-Mediated)

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│  Client      │────▶│  Android App │────▶│  Biometric   │
│  (Chrome/    │     │  (Auth       │     │  Prompt      │
│   Desktop)   │     │   Provider)  │     │              │
└──────────────┘     └──────────────┘     └──────────────┘
       │                    │                    │
       │                    ▼                    │
       │             ┌──────────────┐            │
       │             │  Secure      │◀───────────┘
       │             │  Enclave     │  (biometric success)
       │             └──────────────┘
       │                    │
       │                    ▼ (encrypted master key)
       │             ┌──────────────┐
       └────────────▶│  Local       │
                     │  Decryption  │
                     └──────────────┘
```

The Android app stores the master key encrypted in the Android Keystore, protected by biometric authentication. When biometric succeeds, it releases the encrypted key to the requesting client via secure local channel.

### 4. Sync Architecture

```
Client A                    Server                    Client B
   │                          │                          │
   │  push(encrypted_delta)   │                          │
   ├─────────────────────────▶│                          │
   │                          │  notify(new_version)     │
   │                          ├─────────────────────────▶│
   │                          │                          │
   │                          │  pull(since_version)     │
   │                          │◀─────────────────────────┤
   │                          │                          │
   │                          │  encrypted_delta         │
   │                          ├─────────────────────────▶│
   │                          │                          │
```

**Sync Protocol**:
1. Each change creates a versioned, encrypted delta
2. Server stores encrypted blobs (zero-knowledge)
3. Clients pull deltas since last known version
4. Conflict resolution uses last-write-wins with merge for non-conflicting fields

### 5. Component Architecture by Platform

#### Chrome Extension (Manifest V3)

```
┌─────────────────────────────────────────────────────────┐
│                    Chrome Extension                      │
├─────────────────────────────────────────────────────────┤
│  Service Worker (service-worker.ts)                     │
│  - Crypto operations via WASM                           │
│  - Message routing between components                   │
│  - Vault state management                               │
├─────────────────────────────────────────────────────────┤
│  Content Scripts                                        │
│  - detector.ts: Login form detection                    │
│  - autofill.ts: Credential injection                    │
│  - content.ts: Main content script entry                │
├─────────────────────────────────────────────────────────┤
│  Popup UI (Popup.tsx)                                   │
│  - Quick credential search                              │
│  - Credential display and copy                          │
│  - Lock/unlock controls                                 │
├─────────────────────────────────────────────────────────┤
│  Shared Libraries (lib/)                                │
│  - crypto.ts: WASM crypto wrapper                       │
│  - storage.ts: Chrome storage abstraction               │
└─────────────────────────────────────────────────────────┘
```

#### Desktop Application

```
┌─────────────────────────────────────────────────────────┐
│                  Desktop App (Tauri)                    │
├─────────────────────────────────────────────────────────┤
│  Frontend (React + TypeScript)                          │
│  - UnlockScreen: Master password entry                  │
│  - VaultList: Credential browser with search            │
│  - CredentialForm: Add/edit credentials                 │
│  - PasswordGenerator: Password generation UI            │
│  - SearchBar: Quick credential lookup                   │
├─────────────────────────────────────────────────────────┤
│  Backend (Rust via Tauri)                               │
│  - Crypto core integration (native)                     │
│  - Local storage (encrypted vault file)                 │
│  - Tauri commands for frontend IPC                      │
│  - State management                                     │
└─────────────────────────────────────────────────────────┘
```

#### Android Application

```
┌─────────────────────────────────────────────────────────┐
│                   Android App (Kotlin)                  │
├─────────────────────────────────────────────────────────┤
│  UI Layer (Jetpack Compose)                             │
│  - Vault browser                                        │
│  - Biometric enrollment                                 │
│  - Settings                                             │
├─────────────────────────────────────────────────────────┤
│  Autofill Service                                       │
│  - Android Autofill Framework                           │
│  - Accessibility service (fallback)                     │
├─────────────────────────────────────────────────────────┤
│  Biometric Auth Provider                                │
│  - Android Keystore integration                         │
│  - Secure key release                                   │
│  - Cross-device auth (local network)                    │
├─────────────────────────────────────────────────────────┤
│  Data Layer                                             │
│  - Room database (encrypted)                            │
│  - Crypto core (JNI)                                    │
│  - Sync client                                          │
└─────────────────────────────────────────────────────────┘
```

## Data Flow

### Credential Storage

```
User Input → Validate → Serialize → Encrypt (AES-256-GCM) → Store Locally → Queue Sync
```

### Credential Retrieval

```
Request → Authenticate → Load Encrypted → Decrypt → Deserialize → Display/Auto-fill
```

## Security Model

### Zero-Knowledge Architecture

1. Master password never leaves client
2. Encryption keys derived client-side only
3. Server stores only encrypted blobs
4. Server authenticates users via separate auth token (not master password)

### Key Hierarchy

```
Master Password
       │
       ▼ (Argon2id)
Master Key
       │
       ├──▶ Vault Encryption Key (HKDF)
       │
       ├──▶ Auth Key (HKDF) → Server authentication
       │
       └──▶ Sharing Key (HKDF) → Credential sharing
```

### Threat Mitigations

| Threat | Mitigation |
|--------|------------|
| Server breach | Zero-knowledge; encrypted blobs only |
| Device theft | Auto-lock, PIN/biometric, remote wipe |
| Network interception | TLS 1.3, certificate pinning |
| Memory attacks | Secure memory handling, key zeroization |
| Weak master password | Strength meter, breach database check |

## Technology Stack

| Component | Technology | Status |
|-----------|------------|--------|
| Crypto Core | Rust (argon2, aes-gcm, hkdf, zeroize) | Implemented |
| WASM Bindings | wasm-bindgen, wasm-pack | Implemented |
| Chrome Extension | TypeScript, React, Manifest V3 | Implemented |
| Desktop App | Tauri, React, TypeScript | Implemented |
| Android App | Kotlin, Jetpack Compose | Planned |
| Sync Backend | Go or Rust, PostgreSQL | Planned |
| Local Storage | Encrypted vault file (AES-256-GCM) | Implemented |
| Sync Protocol | HTTPS + WebSocket (notifications) | Planned |

## Deployment

```
┌─────────────────────────────────────────────────────────┐
│                    Cloud Infrastructure                  │
├─────────────────────────────────────────────────────────┤
│  Load Balancer (CloudFlare/AWS ALB)                     │
│         │                                               │
│         ▼                                               │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐     │
│  │  API Server │  │  API Server │  │  API Server │     │
│  │  (Stateless)│  │  (Stateless)│  │  (Stateless)│     │
│  └─────────────┘  └─────────────┘  └─────────────┘     │
│         │                                               │
│         ▼                                               │
│  ┌─────────────────────────────────────────────────┐   │
│  │            PostgreSQL (Primary + Replica)        │   │
│  └─────────────────────────────────────────────────┘   │
│         │                                               │
│         ▼                                               │
│  ┌─────────────────────────────────────────────────┐   │
│  │               S3 / Object Storage                │   │
│  │             (Encrypted vault blobs)              │   │
│  └─────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
```
