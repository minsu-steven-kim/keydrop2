// This file would contain WASM crypto wrapper for web builds
// For Tauri desktop app, we use native Rust crypto via IPC commands

// Re-export types for consistency
export type { VaultItem, PasswordOptions } from '../hooks/useTauri';
