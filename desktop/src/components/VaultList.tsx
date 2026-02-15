import { VaultItem } from '../hooks/useTauri';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import { useState } from 'react';

const icons = {
  key: <path d="M12.65 10A5.99 5.99 0 0 0 7 6c-3.31 0-6 2.69-6 6s2.69 6 6 6a5.99 5.99 0 0 0 5.65-4H17v4h4v-4h2v-4H12.65zM7 14c-1.1 0-2-.9-2-2s.9-2 2-2 2 .9 2 2-.9 2-2 2z"/>,
  star: <path d="M12 17.27L18.18 21l-1.64-7.03L22 9.24l-7.19-.61L12 2 9.19 8.63 2 9.24l5.46 4.73L5.82 21z"/>,
  user: <path d="M12 12c2.21 0 4-1.79 4-4s-1.79-4-4-4-4 1.79-4 4 1.79 4 4 4zm0 2c-2.67 0-8 1.34-8 4v2h16v-2c0-2.66-5.33-4-8-4z"/>,
  password: <path d="M18 8h-1V6c0-2.76-2.24-5-5-5S7 3.24 7 6v2H6c-1.1 0-2 .9-2 2v10c0 1.1.9 2 2 2h12c1.1 0 2-.9 2-2V10c0-1.1-.9-2-2-2zm-6 9c-1.1 0-2-.9-2-2s.9-2 2-2 2 .9 2 2-.9 2-2 2zm3.1-9H8.9V6c0-1.71 1.39-3.1 3.1-3.1 1.71 0 3.1 1.39 3.1 3.1v2z"/>,
  trash: <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z"/>,
  check: <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z"/>,
};

function Icon({ name, size = 18 }: { name: keyof typeof icons; size?: number }) {
  return (
    <svg viewBox="0 0 24 24" fill="currentColor" width={size} height={size}>
      {icons[name]}
    </svg>
  );
}

interface VaultListProps {
  items: VaultItem[];
  onEdit: (item: VaultItem) => void;
  onDelete: (id: string) => void;
}

export default function VaultList({ items, onEdit, onDelete }: VaultListProps) {
  const [copiedId, setCopiedId] = useState<string | null>(null);

  if (items.length === 0) {
    return (
      <div className="empty-state">
        <div className="empty-state-icon"><Icon name="key" size={48} /></div>
        <h3 className="empty-state-title">No credentials yet</h3>
        <p>Click "Add New" to store your first credential</p>
      </div>
    );
  }

  const copyToClipboard = async (text: string, id: string) => {
    await writeText(text);
    setCopiedId(id);
    setTimeout(() => setCopiedId(null), 2000);
  };

  const getInitial = (name: string) => {
    return name.charAt(0).toUpperCase();
  };

  const formatUrl = (url: string | null) => {
    if (!url) return null;
    try {
      const parsed = new URL(url);
      return parsed.hostname;
    } catch {
      return url;
    }
  };

  return (
    <div className="vault-list">
      {items.map((item) => (
        <div key={item.id} className="vault-item" onClick={() => onEdit(item)}>
          <div className="vault-item-icon">
            {getInitial(item.name)}
          </div>
          <div className="vault-item-info">
            <div className="vault-item-name">
              {item.favorite && <><Icon name="star" size={14} />{' '}</>}{item.name}
            </div>
            <div className="vault-item-username">
              {item.username}
              {item.url && ` • ${formatUrl(item.url)}`}
            </div>
          </div>
          <div className="vault-item-actions" onClick={(e) => e.stopPropagation()}>
            <button
              className="btn btn-icon btn-ghost"
              onClick={() => copyToClipboard(item.username, `user-${item.id}`)}
              title="Copy username"
            >
              {copiedId === `user-${item.id}` ? <Icon name="check" /> : <Icon name="user" />}
            </button>
            <button
              className="btn btn-icon btn-ghost"
              onClick={() => copyToClipboard(item.password, `pass-${item.id}`)}
              title="Copy password"
            >
              {copiedId === `pass-${item.id}` ? <Icon name="check" /> : <Icon name="password" />}
            </button>
            <button
              className="btn btn-icon btn-ghost"
              onClick={() => onDelete(item.id)}
              title="Delete"
            >
              <Icon name="trash" />
            </button>
          </div>
        </div>
      ))}
    </div>
  );
}
