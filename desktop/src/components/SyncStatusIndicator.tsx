import { SyncStatus, SyncStatusState } from '../hooks/useTauri';

interface SyncStatusIndicatorProps {
  status: SyncStatus;
  onSyncClick: () => void;
}

const icons = {
  syncing: (
    <svg className="sync-icon spinning" viewBox="0 0 24 24" fill="currentColor" width="18" height="18">
      <path d="M12 4V1L8 5l4 4V6c3.31 0 6 2.69 6 6 0 1.01-.25 1.97-.7 2.8l1.46 1.46C19.54 15.03 20 13.57 20 12c0-4.42-3.58-8-8-8zm0 14c-3.31 0-6-2.69-6-6 0-1.01.25-1.97.7-2.8L5.24 7.74C4.46 8.97 4 10.43 4 12c0 4.42 3.58 8 8 8v3l4-4-4-4v3z"/>
    </svg>
  ),
  synced: (
    <svg className="sync-icon" viewBox="0 0 24 24" fill="currentColor" width="18" height="18">
      <path d="M19.35 10.04C18.67 6.59 15.64 4 12 4 9.11 4 6.6 5.64 5.35 8.04 2.34 8.36 0 10.91 0 14c0 3.31 2.69 6 6 6h13c2.76 0 5-2.24 5-5 0-2.64-2.05-4.78-4.65-4.96zM10 17l-3.5-3.5 1.41-1.41L10 14.17l4.59-4.59L16 11l-6 6z"/>
    </svg>
  ),
  error: (
    <svg className="sync-icon error" viewBox="0 0 24 24" fill="currentColor" width="18" height="18">
      <path d="M19.35 10.04C18.67 6.59 15.64 4 12 4 9.11 4 6.6 5.64 5.35 8.04 2.34 8.36 0 10.91 0 14c0 3.31 2.69 6 6 6h13c2.76 0 5-2.24 5-5 0-2.64-2.05-4.78-4.65-4.96zM12 17l-4-4h3V9h2v4h3l-4 4z"/>
    </svg>
  ),
  offline: (
    <svg className="sync-icon offline" viewBox="0 0 24 24" fill="currentColor" width="18" height="18">
      <path d="M19.35 10.04C18.67 6.59 15.64 4 12 4c-1.48 0-2.85.43-4.01 1.17l1.46 1.46C10.21 6.23 11.08 6 12 6c3.04 0 5.5 2.46 5.5 5.5v.5H19c1.66 0 3 1.34 3 3 0 1.13-.64 2.11-1.56 2.62l1.45 1.45C23.16 18.16 24 16.68 24 15c0-2.64-2.05-4.78-4.65-4.96zM3 5.27l2.75 2.74C2.56 8.15 0 10.77 0 14c0 3.31 2.69 6 6 6h11.73l2 2L21 20.73 4.27 4 3 5.27zM7.73 10l8 8H6c-2.21 0-4-1.79-4-4s1.79-4 4-4h1.73z"/>
    </svg>
  ),
  idle: (
    <svg className="sync-icon" viewBox="0 0 24 24" fill="currentColor" width="18" height="18">
      <path d="M19.35 10.04C18.67 6.59 15.64 4 12 4 9.11 4 6.6 5.64 5.35 8.04 2.34 8.36 0 10.91 0 14c0 3.31 2.69 6 6 6h13c2.76 0 5-2.24 5-5 0-2.64-2.05-4.78-4.65-4.96z"/>
    </svg>
  ),
};

function getStatusLabel(state: SyncStatusState): string {
  switch (state) {
    case 'Syncing':
      return 'Syncing...';
    case 'Error':
      return 'Sync error';
    case 'Offline':
      return 'Offline';
    case 'Idle':
    default:
      return 'Synced';
  }
}

function formatLastSync(timestamp: number | null): string {
  if (!timestamp) return 'Never synced';

  const date = new Date(timestamp * 1000);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffMins = Math.floor(diffMs / 60000);
  const diffHours = Math.floor(diffMins / 60);
  const diffDays = Math.floor(diffHours / 24);

  if (diffMins < 1) return 'Just now';
  if (diffMins < 60) return `${diffMins}m ago`;
  if (diffHours < 24) return `${diffHours}h ago`;
  return `${diffDays}d ago`;
}

export default function SyncStatusIndicator({ status, onSyncClick }: SyncStatusIndicatorProps) {
  const icon = status.state === 'Syncing' ? icons.syncing
    : status.state === 'Error' ? icons.error
    : status.state === 'Offline' ? icons.offline
    : status.last_sync_time ? icons.synced
    : icons.idle;

  return (
    <button
      className={`sync-status-indicator ${status.state.toLowerCase()}`}
      onClick={onSyncClick}
      title={`${getStatusLabel(status.state)}\n${formatLastSync(status.last_sync_time)}${status.error ? `\n${status.error}` : ''}`}
    >
      {icon}
      {status.pending_changes > 0 && (
        <span className="sync-badge">{status.pending_changes}</span>
      )}
    </button>
  );
}
