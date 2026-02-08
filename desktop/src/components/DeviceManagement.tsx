import { useState, useEffect } from 'react';

interface Device {
  id: string;
  name: string;
  type: string;
  lastSeenAt: number;
  isCurrent: boolean;
}

interface DeviceManagementProps {
  onClose: () => void;
}

// Mock API functions - these would be replaced with actual Tauri commands
async function fetchDevices(): Promise<Device[]> {
  // In real implementation, call Tauri backend
  return [
    { id: '1', name: 'Desktop - Windows', type: 'desktop', lastSeenAt: Date.now(), isCurrent: true },
    { id: '2', name: 'Phone - Android', type: 'android', lastSeenAt: Date.now() - 3600000, isCurrent: false },
  ];
}

async function lockDevice(deviceId: string): Promise<void> {
  console.log('Locking device:', deviceId);
}

async function wipeDevice(deviceId: string): Promise<void> {
  console.log('Wiping device:', deviceId);
}

async function deleteDevice(deviceId: string): Promise<void> {
  console.log('Deleting device:', deviceId);
}

function formatLastSeen(timestamp: number): string {
  const now = Date.now();
  const diffMs = now - timestamp;
  const diffMins = Math.floor(diffMs / 60000);
  const diffHours = Math.floor(diffMins / 60);
  const diffDays = Math.floor(diffHours / 24);

  if (diffMins < 1) return 'Just now';
  if (diffMins < 60) return `${diffMins}m ago`;
  if (diffHours < 24) return `${diffHours}h ago`;
  return `${diffDays}d ago`;
}

function getDeviceIcon(type: string): string {
  switch (type.toLowerCase()) {
    case 'android':
      return '📱';
    case 'ios':
      return '📱';
    case 'desktop':
    default:
      return '💻';
  }
}

export default function DeviceManagement({ onClose }: DeviceManagementProps) {
  const [devices, setDevices] = useState<Device[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [actionInProgress, setActionInProgress] = useState<string | null>(null);
  const [confirmAction, setConfirmAction] = useState<{ deviceId: string; action: 'lock' | 'wipe' | 'delete' } | null>(null);

  useEffect(() => {
    loadDevices();
  }, []);

  const loadDevices = async () => {
    setLoading(true);
    try {
      const data = await fetchDevices();
      setDevices(data);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  const handleAction = async (deviceId: string, action: 'lock' | 'wipe' | 'delete') => {
    setActionInProgress(deviceId);
    try {
      switch (action) {
        case 'lock':
          await lockDevice(deviceId);
          break;
        case 'wipe':
          await wipeDevice(deviceId);
          break;
        case 'delete':
          await deleteDevice(deviceId);
          setDevices(devices.filter(d => d.id !== deviceId));
          break;
      }
    } catch (e) {
      setError(String(e));
    } finally {
      setActionInProgress(null);
      setConfirmAction(null);
    }
  };

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-content device-management" onClick={e => e.stopPropagation()}>
        <div className="modal-header">
          <h2>Devices</h2>
          <button className="btn btn-ghost" onClick={onClose}>×</button>
        </div>

        <div className="modal-body">
          {loading ? (
            <div className="loading-small">
              <div className="spinner" />
            </div>
          ) : error ? (
            <div className="error-message">
              {error}
              <button className="btn btn-ghost" onClick={() => setError(null)}>×</button>
            </div>
          ) : devices.length === 0 ? (
            <div className="empty-state">
              <p>No other devices connected</p>
            </div>
          ) : (
            <div className="device-list">
              {devices.map(device => (
                <div key={device.id} className={`device-item ${device.isCurrent ? 'current' : ''}`}>
                  <div className="device-icon">{getDeviceIcon(device.type)}</div>
                  <div className="device-info">
                    <div className="device-name">
                      {device.name}
                      {device.isCurrent && <span className="badge">This device</span>}
                    </div>
                    <div className="device-meta">
                      Last seen: {formatLastSeen(device.lastSeenAt)}
                    </div>
                  </div>
                  {!device.isCurrent && (
                    <div className="device-actions">
                      {actionInProgress === device.id ? (
                        <div className="spinner-small" />
                      ) : (
                        <>
                          <button
                            className="btn btn-icon"
                            title="Lock device"
                            onClick={() => setConfirmAction({ deviceId: device.id, action: 'lock' })}
                          >
                            🔒
                          </button>
                          <button
                            className="btn btn-icon btn-danger"
                            title="Wipe device"
                            onClick={() => setConfirmAction({ deviceId: device.id, action: 'wipe' })}
                          >
                            🗑️
                          </button>
                          <button
                            className="btn btn-icon"
                            title="Remove device"
                            onClick={() => setConfirmAction({ deviceId: device.id, action: 'delete' })}
                          >
                            ✕
                          </button>
                        </>
                      )}
                    </div>
                  )}
                </div>
              ))}
            </div>
          )}
        </div>

        {/* Confirmation Dialog */}
        {confirmAction && (
          <div className="confirm-overlay">
            <div className="confirm-dialog">
              <h3>
                {confirmAction.action === 'lock' && 'Lock Device?'}
                {confirmAction.action === 'wipe' && 'Wipe Device?'}
                {confirmAction.action === 'delete' && 'Remove Device?'}
              </h3>
              <p>
                {confirmAction.action === 'lock' && 'This will lock the vault on the device. They will need to enter their master password to unlock.'}
                {confirmAction.action === 'wipe' && 'This will permanently delete all vault data from the device. This cannot be undone.'}
                {confirmAction.action === 'delete' && 'This will remove the device from your account. They will need to sign in again.'}
              </p>
              <div className="confirm-actions">
                <button className="btn btn-secondary" onClick={() => setConfirmAction(null)}>
                  Cancel
                </button>
                <button
                  className={`btn ${confirmAction.action === 'wipe' ? 'btn-danger' : 'btn-primary'}`}
                  onClick={() => handleAction(confirmAction.deviceId, confirmAction.action)}
                >
                  {confirmAction.action === 'lock' && 'Lock'}
                  {confirmAction.action === 'wipe' && 'Wipe'}
                  {confirmAction.action === 'delete' && 'Remove'}
                </button>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
