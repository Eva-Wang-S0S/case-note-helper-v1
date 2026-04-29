import { useState, useEffect } from 'react';
import { useNavigate, useSearchParams } from 'react-router-dom';
import { useAppStore, Case } from '../stores/appStore';

export function Archive() {
  const navigate = useNavigate();
  const [searchParams, setSearchParams] = useSearchParams();
  const { searchArchive, isLoading, error, setError } = useAppStore();

  const initialQuery = searchParams.get('q') || '';
  const [query, setQuery] = useState(initialQuery);
  const [results, setResults] = useState<Case[]>([]);

  useEffect(() => {
    if (initialQuery) {
      handleSearch(initialQuery);
    }
  }, []);

  const handleSearch = async (searchQuery: string) => {
    if (!searchQuery.trim() && searchQuery !== '') {
      return;
    }

    try {
      const searchResults = await searchArchive(searchQuery);
      setResults(searchResults);
      if (searchQuery) {
        setSearchParams({ q: searchQuery });
      }
    } catch (e) {
      setError(String(e));
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      handleSearch(query);
    }
  };

  return (
    <div className="page-container">
      <h2 style={{ fontSize: '24px', fontWeight: 600, marginBottom: '24px' }}>Archive</h2>

      <div style={{ maxWidth: '600px', marginBottom: '32px' }}>
        <div style={{ display: 'flex', gap: '12px' }}>
          <input
            type="text"
            className="input"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Search cases, notes, keywords..."
            style={{ flex: 1 }}
          />
          <button
            className="btn btn-primary"
            onClick={() => handleSearch(query)}
            disabled={isLoading}
          >
            {isLoading ? <span className="btn-spinner" /> : 'Search'}
          </button>
        </div>
        <p style={{ fontSize: '12px', color: 'var(--color-text-muted)', marginTop: '8px' }}>
          Press Enter to search
        </p>
      </div>

      {error && (
        <div className="error-banner">
          <span className="error-banner-message">{error}</span>
          <button className="btn btn-ghost" onClick={() => setError(null)}>Dismiss</button>
        </div>
      )}

      {results.length > 0 ? (
        <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
          <p style={{ fontSize: '14px', color: 'var(--color-text-secondary)', marginBottom: '8px' }}>
            {results.length} result{results.length !== 1 ? 's' : ''} found
          </p>
          {results.map((c) => (
            <div
              key={c.id}
              onClick={() => navigate(`/case/${c.id}/notes`)}
              style={{
                padding: '16px',
                background: 'var(--color-surface)',
                border: '1px solid var(--color-border)',
                borderRadius: 'var(--radius-md)',
                cursor: 'pointer',
              }}
            >
              <div style={{ fontWeight: 600 }}>{c.name}</div>
              <div style={{ fontSize: '13px', color: 'var(--color-text-secondary)', marginTop: '4px' }}>
                {c.client_name} &middot; {c.stage}
              </div>
            </div>
          ))}
        </div>
      ) : query && !isLoading ? (
        <div className="empty-state" style={{ padding: '48px' }}>
          <p className="empty-state-description">No cases match your search</p>
        </div>
      ) : !query ? (
        <div className="empty-state" style={{ padding: '48px' }}>
          <p className="empty-state-description">
            Enter a search query to find archived cases
          </p>
        </div>
      ) : null}
    </div>
  );
}