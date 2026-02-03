function SearchIcon() {
  return (
    <svg viewBox="0 0 24 24" fill="currentColor" width="18" height="18">
      <path d="M15.5 14h-.79l-.28-.27A6.471 6.471 0 0016 9.5 6.5 6.5 0 109.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5 14 7.01 14 9.5 11.99 14 9.5 14z"/>
    </svg>
  );
}

interface SearchBarProps {
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
}

export default function SearchBar({ value, onChange, placeholder = 'Search...' }: SearchBarProps) {
  return (
    <div className="input-with-icon" style={{ width: '300px' }}>
      <input
        type="text"
        className="input"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder={placeholder}
        style={{ paddingLeft: '36px' }}
      />
      <span className="input-icon" style={{ left: '10px', right: 'auto', pointerEvents: 'none' }}>
        <SearchIcon />
      </span>
      {value && (
        <span
          className="input-icon"
          onClick={() => onChange('')}
          style={{ cursor: 'pointer' }}
        >
          ×
        </span>
      )}
    </div>
  );
}
