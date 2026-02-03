import { useState, useEffect, useCallback } from 'react';
import { tauri, VaultItem, VaultStatus } from './useTauri';

export function useVault() {
  const [status, setStatus] = useState<VaultStatus | null>(null);
  const [items, setItems] = useState<VaultItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const refreshStatus = useCallback(async () => {
    try {
      const newStatus = await tauri.getVaultStatus();
      setStatus(newStatus);
      return newStatus;
    } catch (err) {
      setError(String(err));
      return null;
    }
  }, []);

  const refreshItems = useCallback(async () => {
    try {
      const newItems = await tauri.getAllItems();
      setItems(newItems);
    } catch (err) {
      setError(String(err));
    }
  }, []);

  useEffect(() => {
    const init = async () => {
      setLoading(true);
      const newStatus = await refreshStatus();
      if (newStatus?.unlocked) {
        await refreshItems();
      }
      setLoading(false);
    };
    init();
  }, [refreshStatus, refreshItems]);

  // Auto-lock check interval
  useEffect(() => {
    if (!status?.unlocked) return;

    const interval = setInterval(async () => {
      const locked = await tauri.checkAutoLock();
      if (locked) {
        setStatus((prev) => (prev ? { ...prev, unlocked: false } : null));
        setItems([]);
      }
    }, 10000);

    return () => clearInterval(interval);
  }, [status?.unlocked]);

  const createVault = async (password: string) => {
    setError(null);
    try {
      await tauri.createVault(password);
      await refreshStatus();
      await refreshItems();
    } catch (err) {
      setError(String(err));
      throw err;
    }
  };

  const unlock = async (password: string) => {
    setError(null);
    try {
      await tauri.unlockVault(password);
      await refreshStatus();
      await refreshItems();
    } catch (err) {
      setError(String(err));
      throw err;
    }
  };

  const lock = async () => {
    try {
      await tauri.lockVault();
      setStatus((prev) => (prev ? { ...prev, unlocked: false } : null));
      setItems([]);
    } catch (err) {
      setError(String(err));
    }
  };

  const addItem = async (item: Omit<VaultItem, 'id' | 'created_at' | 'modified_at'>) => {
    setError(null);
    try {
      const now = Math.floor(Date.now() / 1000);
      const fullItem: VaultItem = {
        ...item,
        id: '',
        created_at: now,
        modified_at: now,
      };
      await tauri.addItem(fullItem);
      await refreshItems();
    } catch (err) {
      setError(String(err));
      throw err;
    }
  };

  const updateItem = async (id: string, item: VaultItem) => {
    setError(null);
    try {
      await tauri.updateItem(id, item);
      await refreshItems();
    } catch (err) {
      setError(String(err));
      throw err;
    }
  };

  const deleteItem = async (id: string) => {
    setError(null);
    try {
      await tauri.deleteItem(id);
      await refreshItems();
    } catch (err) {
      setError(String(err));
      throw err;
    }
  };

  const search = async (query: string): Promise<VaultItem[]> => {
    if (!query.trim()) {
      return items;
    }
    try {
      return await tauri.searchItems(query);
    } catch (err) {
      setError(String(err));
      return [];
    }
  };

  return {
    status,
    items,
    loading,
    error,
    createVault,
    unlock,
    lock,
    addItem,
    updateItem,
    deleteItem,
    search,
    refreshItems,
    clearError: () => setError(null),
  };
}
