import React, { useState, useEffect } from 'react';
import ReactDOM from 'react-dom/client';

interface VaultItem {
  id: string;
  name: string;
  url: string | null;
  username: string;
  password: string;
}

interface VaultStatus {
  vaultExists: boolean;
  unlocked: boolean;
  itemCount: number;
}

// Styles
const styles = {
  container: {
    padding: '16px',
    minHeight: '400px',
    display: 'flex',
    flexDirection: 'column' as const,
  },
  header: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    marginBottom: '16px',
    paddingBottom: '12px',
    borderBottom: '1px solid #333',
  },
  logo: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
    fontSize: '18px',
    fontWeight: 'bold' as const,
    color: '#e94560',
  },
  lockBtn: {
    padding: '6px 12px',
    background: '#333',
    color: '#eee',
    border: 'none',
    borderRadius: '4px',
    cursor: 'pointer',
    fontSize: '12px',
  },
  searchInput: {
    width: '100%',
    padding: '10px 12px',
    background: '#16213e',
    border: '1px solid #333',
    borderRadius: '6px',
    color: '#eee',
    fontSize: '14px',
    marginBottom: '12px',
  },
  list: {
    flex: 1,
    overflowY: 'auto' as const,
  },
  item: {
    display: 'flex',
    alignItems: 'center',
    gap: '12px',
    padding: '12px',
    background: '#16213e',
    borderRadius: '6px',
    marginBottom: '8px',
    cursor: 'pointer',
    border: '1px solid transparent',
    transition: 'border-color 0.2s',
  },
  itemIcon: {
    width: '36px',
    height: '36px',
    background: '#0f3460',
    borderRadius: '6px',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    fontSize: '16px',
  },
  itemInfo: {
    flex: 1,
    minWidth: 0,
  },
  itemName: {
    fontWeight: 500 as const,
    marginBottom: '2px',
    whiteSpace: 'nowrap' as const,
    overflow: 'hidden',
    textOverflow: 'ellipsis',
  },
  itemUsername: {
    fontSize: '12px',
    color: '#aaa',
    whiteSpace: 'nowrap' as const,
    overflow: 'hidden',
    textOverflow: 'ellipsis',
  },
  itemActions: {
    display: 'flex',
    gap: '4px',
  },
  iconBtn: {
    width: '28px',
    height: '28px',
    background: 'transparent',
    border: 'none',
    borderRadius: '4px',
    cursor: 'pointer',
    fontSize: '14px',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
  },
  unlockForm: {
    display: 'flex',
    flexDirection: 'column' as const,
    alignItems: 'center',
    justifyContent: 'center',
    flex: 1,
    padding: '20px',
  },
  unlockTitle: {
    fontSize: '24px',
    marginBottom: '8px',
    color: '#e94560',
  },
  unlockSubtitle: {
    color: '#aaa',
    marginBottom: '20px',
    textAlign: 'center' as const,
  },
  input: {
    width: '100%',
    padding: '12px',
    background: '#16213e',
    border: '1px solid #333',
    borderRadius: '6px',
    color: '#eee',
    fontSize: '14px',
    marginBottom: '12px',
  },
  btn: {
    width: '100%',
    padding: '12px',
    background: '#e94560',
    color: 'white',
    border: 'none',
    borderRadius: '6px',
    cursor: 'pointer',
    fontSize: '14px',
    fontWeight: 500 as const,
  },
  error: {
    background: 'rgba(239, 68, 68, 0.1)',
    border: '1px solid #ef4444',
    color: '#ef4444',
    padding: '10px',
    borderRadius: '6px',
    marginBottom: '12px',
    fontSize: '14px',
  },
  empty: {
    textAlign: 'center' as const,
    padding: '40px 20px',
    color: '#aaa',
  },
};

function Popup() {
  const [status, setStatus] = useState<VaultStatus | null>(null);
  const [items, setItems] = useState<VaultItem[]>([]);
  const [password, setPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(true);
  const [searchQuery, setSearchQuery] = useState('');
  const [copiedId, setCopiedId] = useState<string | null>(null);

  useEffect(() => {
    loadStatus();
  }, []);

  const loadStatus = async () => {
    setLoading(true);
    const response = await chrome.runtime.sendMessage({ type: 'GET_STATUS' });
    if (response.success) {
      setStatus(response.data);
      if (response.data.unlocked) {
        await loadItems();
      }
    }
    setLoading(false);
  };

  const loadItems = async () => {
    const response = await chrome.runtime.sendMessage({ type: 'GET_ITEMS' });
    if (response.success) {
      setItems(response.data);
    }
  };

  const handleUnlock = async (e: React.FormEvent) => {
    e.preventDefault();
    setError('');

    const response = await chrome.runtime.sendMessage({
      type: 'UNLOCK',
      password,
    });

    if (response.success) {
      setPassword('');
      await loadStatus();
    } else {
      setError(response.error || 'Failed to unlock');
    }
  };

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    setError('');

    if (password !== confirmPassword) {
      setError('Passwords do not match');
      return;
    }

    if (password.length < 8) {
      setError('Password must be at least 8 characters');
      return;
    }

    const response = await chrome.runtime.sendMessage({
      type: 'CREATE_VAULT',
      password,
    });

    if (response.success) {
      setPassword('');
      setConfirmPassword('');
      await loadStatus();
    } else {
      setError(response.error || 'Failed to create vault');
    }
  };

  const handleLock = async () => {
    await chrome.runtime.sendMessage({ type: 'LOCK' });
    setItems([]);
    await loadStatus();
  };

  const copyToClipboard = async (text: string, id: string) => {
    await navigator.clipboard.writeText(text);
    setCopiedId(id);
    setTimeout(() => setCopiedId(null), 2000);
  };

  const handleAutofill = async (item: VaultItem) => {
    const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
    if (tab?.id) {
      await chrome.runtime.sendMessage({
        type: 'AUTOFILL',
        tabId: tab.id,
        item,
      });
      window.close();
    }
  };

  const filteredItems = items.filter(
    (item) =>
      item.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      item.username.toLowerCase().includes(searchQuery.toLowerCase())
  );

  if (loading) {
    return (
      <div style={{ ...styles.container, justifyContent: 'center', alignItems: 'center' }}>
        <div>Loading...</div>
      </div>
    );
  }

  if (!status?.vaultExists) {
    return (
      <div style={styles.container}>
        <div style={styles.unlockForm}>
          <div style={styles.unlockTitle}>🔐 Keydrop</div>
          <div style={styles.unlockSubtitle}>Create a master password to get started</div>

          {error && <div style={styles.error}>{error}</div>}

          <form onSubmit={handleCreate} style={{ width: '100%' }}>
            <input
              type="password"
              style={styles.input}
              placeholder="Master password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              autoFocus
            />
            <input
              type="password"
              style={styles.input}
              placeholder="Confirm password"
              value={confirmPassword}
              onChange={(e) => setConfirmPassword(e.target.value)}
            />
            <button type="submit" style={styles.btn}>
              Create Vault
            </button>
          </form>
        </div>
      </div>
    );
  }

  if (!status.unlocked) {
    return (
      <div style={styles.container}>
        <div style={styles.unlockForm}>
          <div style={styles.unlockTitle}>🔐 Keydrop</div>
          <div style={styles.unlockSubtitle}>Enter your master password to unlock</div>

          {error && <div style={styles.error}>{error}</div>}

          <form onSubmit={handleUnlock} style={{ width: '100%' }}>
            <input
              type="password"
              style={styles.input}
              placeholder="Master password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              autoFocus
            />
            <button type="submit" style={styles.btn}>
              Unlock
            </button>
          </form>
        </div>
      </div>
    );
  }

  return (
    <div style={styles.container}>
      <div style={styles.header}>
        <div style={styles.logo}>
          <span>🔐</span>
          <span>Keydrop</span>
        </div>
        <button style={styles.lockBtn} onClick={handleLock}>
          Lock
        </button>
      </div>

      <input
        type="text"
        style={styles.searchInput}
        placeholder="Search credentials..."
        value={searchQuery}
        onChange={(e) => setSearchQuery(e.target.value)}
      />

      <div style={styles.list}>
        {filteredItems.length === 0 ? (
          <div style={styles.empty}>
            {searchQuery ? 'No matching credentials' : 'No credentials yet'}
          </div>
        ) : (
          filteredItems.map((item) => (
            <div
              key={item.id}
              style={styles.item}
              onClick={() => handleAutofill(item)}
              onMouseEnter={(e) => {
                e.currentTarget.style.borderColor = '#e94560';
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.borderColor = 'transparent';
              }}
            >
              <div style={styles.itemIcon}>{item.name.charAt(0).toUpperCase()}</div>
              <div style={styles.itemInfo}>
                <div style={styles.itemName}>{item.name}</div>
                <div style={styles.itemUsername}>{item.username}</div>
              </div>
              <div style={styles.itemActions} onClick={(e) => e.stopPropagation()}>
                <button
                  style={styles.iconBtn}
                  onClick={() => copyToClipboard(item.username, `user-${item.id}`)}
                  title="Copy username"
                >
                  {copiedId === `user-${item.id}` ? '✓' : '👤'}
                </button>
                <button
                  style={styles.iconBtn}
                  onClick={() => copyToClipboard(item.password, `pass-${item.id}`)}
                  title="Copy password"
                >
                  {copiedId === `pass-${item.id}` ? '✓' : '🔑'}
                </button>
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
}

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <Popup />
  </React.StrictMode>
);
