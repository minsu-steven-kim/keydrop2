import { useState } from 'react';

function KeyIcon() {
  return (
    <svg viewBox="0 0 24 24" fill="currentColor" width="32" height="32" style={{ marginRight: '8px', verticalAlign: 'middle' }}>
      <path d="M12.65 10A5.99 5.99 0 0 0 7 6c-3.31 0-6 2.69-6 6s2.69 6 6 6a5.99 5.99 0 0 0 5.65-4H17v4h4v-4h2v-4H12.65zM7 14c-1.1 0-2-.9-2-2s.9-2 2-2 2 .9 2 2-.9 2-2 2z"/>
    </svg>
  );
}

function EyeIcon({ open }: { open: boolean }) {
  if (open) {
    return (
      <svg viewBox="0 0 24 24" fill="currentColor" width="20" height="20">
        <path d="M12 4.5C7 4.5 2.73 7.61 1 12c1.73 4.39 6 7.5 11 7.5s9.27-3.11 11-7.5c-1.73-4.39-6-7.5-11-7.5zM12 17c-2.76 0-5-2.24-5-5s2.24-5 5-5 5 2.24 5 5-2.24 5-5 5zm0-8c-1.66 0-3 1.34-3 3s1.34 3 3 3 3-1.34 3-3-1.34-3-3-3z"/>
      </svg>
    );
  }
  return (
    <svg viewBox="0 0 24 24" fill="currentColor" width="20" height="20">
      <path d="M12 7c2.76 0 5 2.24 5 5 0 .65-.13 1.26-.36 1.83l2.92 2.92c1.51-1.26 2.7-2.89 3.43-4.75-1.73-4.39-6-7.5-11-7.5-1.4 0-2.74.25-3.98.7l2.16 2.16C10.74 7.13 11.35 7 12 7zM2 4.27l2.28 2.28.46.46C3.08 8.3 1.78 10.02 1 12c1.73 4.39 6 7.5 11 7.5 1.55 0 3.03-.3 4.38-.84l.42.42L19.73 22 21 20.73 3.27 3 2 4.27zM7.53 9.8l1.55 1.55c-.05.21-.08.43-.08.65 0 1.66 1.34 3 3 3 .22 0 .44-.03.65-.08l1.55 1.55c-.67.33-1.41.53-2.2.53-2.76 0-5-2.24-5-5 0-.79.2-1.53.53-2.2zm4.31-.78l3.15 3.15.02-.16c0-1.66-1.34-3-3-3l-.17.01z"/>
    </svg>
  );
}

interface UnlockScreenProps {
  hasVault: boolean;
  onUnlock: (password: string) => Promise<void>;
  onCreate: (password: string) => Promise<void>;
  error: string | null;
  onClearError: () => void;
}

export default function UnlockScreen({
  hasVault,
  onUnlock,
  onCreate,
  error,
  onClearError,
}: UnlockScreenProps) {
  const [password, setPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [showPassword, setShowPassword] = useState(false);
  const [loading, setLoading] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    onClearError();

    if (!hasVault && password !== confirmPassword) {
      return;
    }

    setLoading(true);
    try {
      if (hasVault) {
        await onUnlock(password);
      } else {
        await onCreate(password);
      }
    } finally {
      setLoading(false);
    }
  };

  const isValid = password.length >= 8 && (hasVault || password === confirmPassword);

  return (
    <div className="unlock-screen">
      <div className="unlock-card">
        <h1 className="unlock-title"><KeyIcon /> Keydrop</h1>
        <p className="unlock-subtitle">
          {hasVault ? 'Enter your master password to unlock' : 'Create a master password to get started'}
        </p>

        {error && <div className="error-message">{error}</div>}

        <form className="unlock-form" onSubmit={handleSubmit}>
          <div className="input-group">
            <label className="input-label">Master Password</label>
            <div className="input-with-icon">
              <input
                type={showPassword ? 'text' : 'password'}
                className="input"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                placeholder="Enter master password"
                autoFocus
                minLength={8}
              />
              <span
                className="input-icon"
                onClick={() => setShowPassword(!showPassword)}
              >
                <EyeIcon open={showPassword} />
              </span>
            </div>
          </div>

          {!hasVault && (
            <div className="input-group">
              <label className="input-label">Confirm Password</label>
              <input
                type={showPassword ? 'text' : 'password'}
                className="input"
                value={confirmPassword}
                onChange={(e) => setConfirmPassword(e.target.value)}
                placeholder="Confirm master password"
              />
              {password && confirmPassword && password !== confirmPassword && (
                <small style={{ color: 'var(--error)', marginTop: '4px', display: 'block' }}>
                  Passwords do not match
                </small>
              )}
            </div>
          )}

          {!hasVault && (
            <p style={{ fontSize: '14px', color: 'var(--text-secondary)', marginBottom: '16px' }}>
              Password must be at least 8 characters. This password cannot be recovered if lost.
            </p>
          )}

          <button
            type="submit"
            className="btn btn-primary"
            style={{ width: '100%' }}
            disabled={!isValid || loading}
          >
            {loading ? 'Please wait...' : hasVault ? 'Unlock Vault' : 'Create Vault'}
          </button>
        </form>
      </div>
    </div>
  );
}
