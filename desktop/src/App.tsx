import { useState, useEffect, useCallback } from 'react';
import { useVault } from './hooks/useVault';
import { useSync } from './hooks/useSync';
import { VaultItem, RemoteCommand, tauri } from './hooks/useTauri';
import UnlockScreen from './components/UnlockScreen';
import VaultList from './components/VaultList';
import CredentialForm from './components/CredentialForm';
import SearchBar from './components/SearchBar';
import SyncStatusIndicator from './components/SyncStatusIndicator';

const icons = {
  list: <path d="M3 13h2v-2H3v2zm0 4h2v-2H3v2zm0-8h2V7H3v2zm4 4h14v-2H7v2zm0 4h14v-2H7v2zM7 7v2h14V7H7z"/>,
  star: <path d="M12 17.27L18.18 21l-1.64-7.03L22 9.24l-7.19-.61L12 2 9.19 8.63 2 9.24l5.46 4.73L5.82 21z"/>,
  plus: <path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z"/>,
  lock: <path d="M18 8h-1V6c0-2.76-2.24-5-5-5S7 3.24 7 6v2H6c-1.1 0-2 .9-2 2v10c0 1.1.9 2 2 2h12c1.1 0 2-.9 2-2V10c0-1.1-.9-2-2-2zm-6 9c-1.1 0-2-.9-2-2s.9-2 2-2 2 .9 2 2-.9 2-2 2zm3.1-9H8.9V6c0-1.71 1.39-3.1 3.1-3.1 1.71 0 3.1 1.39 3.1 3.1v2z"/>,
};

function Icon({ name }: { name: keyof typeof icons }) {
  return (
    <svg viewBox="0 0 24 24" fill="currentColor" width="18" height="18" style={{ marginRight: '8px', verticalAlign: 'middle' }}>
      {icons[name]}
    </svg>
  );
}

type View = 'all' | 'favorites' | 'generator';

function App() {
  const {
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
    clearError,
  } = useVault();

  const handleRemoteCommand = useCallback(async (command: RemoteCommand) => {
    if (command.command_type === 'lock') {
      await lock();
    } else if (command.command_type === 'wipe') {
      await tauri.wipeVault();
      window.location.reload();
    }
  }, [lock]);

  const sync = useSync(handleRemoteCommand);

  const [view, setView] = useState<View>('all');
  const [searchQuery, setSearchQuery] = useState('');
  const [searchResults, setSearchResults] = useState<VaultItem[] | null>(null);
  const [showForm, setShowForm] = useState(false);
  const [editingItem, setEditingItem] = useState<VaultItem | null>(null);

  useEffect(() => {
    const performSearch = async () => {
      if (searchQuery.trim()) {
        const results = await search(searchQuery);
        setSearchResults(results);
      } else {
        setSearchResults(null);
      }
    };
    performSearch();
  }, [searchQuery, search]);

  if (loading) {
    return (
      <div className="loading">
        <div className="spinner" />
      </div>
    );
  }

  if (!status?.unlocked) {
    return (
      <UnlockScreen
        hasVault={status?.exists ?? false}
        onUnlock={unlock}
        onCreate={createVault}
        error={error}
        onClearError={clearError}
      />
    );
  }

  const displayedItems = searchResults ?? (view === 'favorites' ? items.filter(i => i.favorite) : items);

  const handleEdit = (item: VaultItem) => {
    setEditingItem(item);
    setShowForm(true);
  };

  const handleAdd = () => {
    setEditingItem(null);
    setShowForm(true);
  };

  const handleSave = async (item: Omit<VaultItem, 'id' | 'created_at' | 'modified_at'>) => {
    if (editingItem) {
      await updateItem(editingItem.id, {
        ...editingItem,
        ...item,
        modified_at: Math.floor(Date.now() / 1000),
      });
    } else {
      await addItem(item);
    }
    setShowForm(false);
    setEditingItem(null);
  };

  const handleDelete = async (id: string) => {
    if (confirm('Are you sure you want to delete this item?')) {
      await deleteItem(id);
    }
  };

  return (
    <div className="app-container">
      <aside className="sidebar">
        <div className="logo">
          <svg className="logo-icon" viewBox="0 0 24 24" fill="currentColor" width="24" height="24">
            <path d="M12.65 10A5.99 5.99 0 0 0 7 6c-3.31 0-6 2.69-6 6s2.69 6 6 6a5.99 5.99 0 0 0 5.65-4H17v4h4v-4h2v-4H12.65zM7 14c-1.1 0-2-.9-2-2s.9-2 2-2 2 .9 2 2-.9 2-2 2z"/>
          </svg>
          Keydrop
        </div>

        <nav>
          <div className="nav-section">
            <div className="nav-section-title">Vault</div>
            <div
              className={`nav-item ${view === 'all' ? 'active' : ''}`}
              onClick={() => { setView('all'); setSearchQuery(''); }}
            >
              <Icon name="list" /> All Items ({items.length})
            </div>
            <div
              className={`nav-item ${view === 'favorites' ? 'active' : ''}`}
              onClick={() => { setView('favorites'); setSearchQuery(''); }}
            >
              <Icon name="star" /> Favorites ({items.filter(i => i.favorite).length})
            </div>
          </div>

          <div className="nav-section">
            <div className="nav-section-title">Actions</div>
            <div className="nav-item" onClick={handleAdd}>
              <Icon name="plus" /> Add New
            </div>
            <div className="nav-item" onClick={lock}>
              <Icon name="lock" /> Lock Vault
            </div>
          </div>
        </nav>
      </aside>

      <main className="main-content">
        <div className="vault-header">
          <h1 className="vault-title">
            {view === 'favorites' ? 'Favorites' : 'All Items'}
          </h1>
          <div className="vault-header-actions">
            <SyncStatusIndicator
              status={sync.status}
              onSyncClick={sync.triggerSync}
            />
            <SearchBar
              value={searchQuery}
              onChange={setSearchQuery}
              placeholder="Search vault..."
            />
          </div>
        </div>

        {error && (
          <div className="error-message">
            {error}
            <button className="btn btn-ghost" onClick={clearError}>×</button>
          </div>
        )}

        <VaultList
          items={displayedItems}
          onEdit={handleEdit}
          onDelete={handleDelete}
        />

        {showForm && (
          <CredentialForm
            item={editingItem}
            onSave={handleSave}
            onCancel={() => { setShowForm(false); setEditingItem(null); }}
          />
        )}
      </main>
    </div>
  );
}

export default App;
