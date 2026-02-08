import { useState, useEffect, useCallback } from 'react';
import { tauri, SyncStatus, RemoteCommand } from './useTauri';

export interface UseSyncResult {
  status: SyncStatus;
  isEnabled: boolean;
  isSyncing: boolean;
  lastSyncTime: Date | null;
  error: string | null;
  pendingChanges: number;
  triggerSync: () => Promise<void>;
  enable: (serverUrl: string, accessToken: string, deviceId: string) => Promise<void>;
  disable: () => Promise<void>;
}

export function useSync(onRemoteCommand?: (command: RemoteCommand) => void): UseSyncResult {
  const [status, setStatus] = useState<SyncStatus>({
    state: 'Idle',
    last_sync_time: null,
    error: null,
    pending_changes: 0,
  });
  const [isEnabled, setIsEnabled] = useState(false);

  const refreshStatus = useCallback(async () => {
    try {
      const newStatus = await tauri.getSyncStatus();
      setStatus(newStatus);
    } catch (err) {
      console.error('Failed to get sync status:', err);
    }
  }, []);

  const checkCommands = useCallback(async () => {
    if (!isEnabled || !onRemoteCommand) return;

    try {
      const commands = await tauri.checkRemoteCommands();
      for (const command of commands) {
        onRemoteCommand(command);
      }
    } catch (err) {
      console.error('Failed to check remote commands:', err);
    }
  }, [isEnabled, onRemoteCommand]);

  // Initial status fetch
  useEffect(() => {
    refreshStatus();
  }, [refreshStatus]);

  // Poll for status updates when enabled
  useEffect(() => {
    if (!isEnabled) return;

    const interval = setInterval(() => {
      refreshStatus();
      checkCommands();
    }, 30000); // Check every 30 seconds

    return () => clearInterval(interval);
  }, [isEnabled, refreshStatus, checkCommands]);

  const triggerSync = useCallback(async () => {
    try {
      setStatus(prev => ({ ...prev, state: 'Syncing' }));
      await tauri.triggerSync();
      await refreshStatus();
    } catch (err) {
      setStatus(prev => ({
        ...prev,
        state: 'Error',
        error: String(err),
      }));
    }
  }, [refreshStatus]);

  const enable = useCallback(async (serverUrl: string, accessToken: string, deviceId: string) => {
    try {
      await tauri.enableSync({
        server_url: serverUrl,
        access_token: accessToken,
        device_id: deviceId,
      });
      setIsEnabled(true);
      await triggerSync();
    } catch (err) {
      console.error('Failed to enable sync:', err);
      throw err;
    }
  }, [triggerSync]);

  const disable = useCallback(async () => {
    try {
      await tauri.disableSync();
      setIsEnabled(false);
      setStatus({
        state: 'Idle',
        last_sync_time: null,
        error: null,
        pending_changes: 0,
      });
    } catch (err) {
      console.error('Failed to disable sync:', err);
      throw err;
    }
  }, []);

  return {
    status,
    isEnabled,
    isSyncing: status.state === 'Syncing',
    lastSyncTime: status.last_sync_time ? new Date(status.last_sync_time * 1000) : null,
    error: status.error,
    pendingChanges: status.pending_changes,
    triggerSync,
    enable,
    disable,
  };
}
