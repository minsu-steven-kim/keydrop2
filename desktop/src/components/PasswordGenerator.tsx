import { useState, useEffect } from 'react';
import { tauri } from '../hooks/useTauri';

const icons = {
  refresh: <path d="M17.65 6.35A7.958 7.958 0 0012 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08A5.99 5.99 0 0112 18c-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z"/>,
  copy: <path d="M16 1H4c-1.1 0-2 .9-2 2v14h2V3h12V1zm3 4H8c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h11c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm0 16H8V7h11v14z"/>,
  check: <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z"/>,
};

function Icon({ name }: { name: keyof typeof icons }) {
  return (
    <svg viewBox="0 0 24 24" fill="currentColor" width="16" height="16" style={{ marginRight: '6px', verticalAlign: 'middle' }}>
      {icons[name]}
    </svg>
  );
}

interface PasswordGeneratorProps {
  onSelect: (password: string) => void;
}

export default function PasswordGenerator({ onSelect }: PasswordGeneratorProps) {
  const [password, setPassword] = useState('');
  const [length, setLength] = useState(16);
  const [lowercase, setLowercase] = useState(true);
  const [uppercase, setUppercase] = useState(true);
  const [digits, setDigits] = useState(true);
  const [symbols, setSymbols] = useState(true);
  const [excludeAmbiguous, setExcludeAmbiguous] = useState(false);
  const [mode, setMode] = useState<'password' | 'passphrase'>('password');
  const [wordCount, setWordCount] = useState(4);
  const [separator, setSeparator] = useState('-');

  const generate = async () => {
    try {
      let newPassword: string;
      if (mode === 'password') {
        newPassword = await tauri.generatePassword({
          length,
          lowercase,
          uppercase,
          digits,
          symbols,
          exclude_ambiguous: excludeAmbiguous,
        });
      } else {
        newPassword = await tauri.generatePassphrase(wordCount, separator);
      }
      setPassword(newPassword);
    } catch (err) {
      console.error('Failed to generate password:', err);
    }
  };

  useEffect(() => {
    generate();
  }, [length, lowercase, uppercase, digits, symbols, excludeAmbiguous, mode, wordCount, separator]);

  const copyToClipboard = async () => {
    await navigator.clipboard.writeText(password);
  };

  return (
    <div className="password-generator">
      <div style={{ display: 'flex', gap: '10px', marginBottom: '16px' }}>
        <button
          type="button"
          className={`btn ${mode === 'password' ? 'btn-primary' : 'btn-secondary'}`}
          onClick={() => setMode('password')}
        >
          Password
        </button>
        <button
          type="button"
          className={`btn ${mode === 'passphrase' ? 'btn-primary' : 'btn-secondary'}`}
          onClick={() => setMode('passphrase')}
        >
          Passphrase
        </button>
      </div>

      <div className="password-display" style={{ fontFamily: 'monospace' }}>
        {password}
      </div>

      <div style={{ display: 'flex', gap: '10px', marginBottom: '16px' }}>
        <button type="button" className="btn btn-secondary" onClick={generate}>
          <Icon name="refresh" /> Regenerate
        </button>
        <button type="button" className="btn btn-secondary" onClick={copyToClipboard}>
          <Icon name="copy" /> Copy
        </button>
        <button type="button" className="btn btn-primary" onClick={() => onSelect(password)}>
          <Icon name="check" /> Use This
        </button>
      </div>

      {mode === 'password' ? (
        <>
          <div className="password-length">
            <label className="input-label">Length: {length}</label>
            <input
              type="range"
              className="password-length-slider"
              min="8"
              max="64"
              value={length}
              onChange={(e) => setLength(Number(e.target.value))}
            />
            <div className="password-length-value">
              <span>8</span>
              <span>64</span>
            </div>
          </div>

          <div className="password-options">
            <label className="password-option">
              <input
                type="checkbox"
                checked={lowercase}
                onChange={(e) => setLowercase(e.target.checked)}
              />
              Lowercase (a-z)
            </label>
            <label className="password-option">
              <input
                type="checkbox"
                checked={uppercase}
                onChange={(e) => setUppercase(e.target.checked)}
              />
              Uppercase (A-Z)
            </label>
            <label className="password-option">
              <input
                type="checkbox"
                checked={digits}
                onChange={(e) => setDigits(e.target.checked)}
              />
              Numbers (0-9)
            </label>
            <label className="password-option">
              <input
                type="checkbox"
                checked={symbols}
                onChange={(e) => setSymbols(e.target.checked)}
              />
              Symbols (!@#$...)
            </label>
            <label className="password-option">
              <input
                type="checkbox"
                checked={excludeAmbiguous}
                onChange={(e) => setExcludeAmbiguous(e.target.checked)}
              />
              Exclude ambiguous (0OlI1)
            </label>
          </div>
        </>
      ) : (
        <>
          <div className="password-length">
            <label className="input-label">Words: {wordCount}</label>
            <input
              type="range"
              className="password-length-slider"
              min="3"
              max="10"
              value={wordCount}
              onChange={(e) => setWordCount(Number(e.target.value))}
            />
            <div className="password-length-value">
              <span>3</span>
              <span>10</span>
            </div>
          </div>

          <div className="input-group">
            <label className="input-label">Separator</label>
            <select
              className="input"
              value={separator}
              onChange={(e) => setSeparator(e.target.value)}
            >
              <option value="-">Hyphen (-)</option>
              <option value="_">Underscore (_)</option>
              <option value=".">Period (.)</option>
              <option value=" ">Space ( )</option>
            </select>
          </div>
        </>
      )}
    </div>
  );
}
