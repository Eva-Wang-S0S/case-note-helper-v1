import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useAppStore } from '../stores/appStore';
import { TodoistPanel } from './TodoistPanel';

interface LayoutProps {
  children: React.ReactNode;
}

export function Layout({ children }: LayoutProps) {
  const [sidebarOpen, setSidebarOpen] = useState(false);
  const navigate = useNavigate();
  const { cases, selectedCaseId, todoistPanelOpen, toggleTodoistPanel } = useAppStore();

  const isMobile = typeof window !== 'undefined' && window.innerWidth < 1024;

  const handleCaseSelect = (caseId: number) => {
    navigate(`/case/${caseId}/notes`);
    setSidebarOpen(false);
  };

  return (
    <div className="app-layout">
      {isMobile && sidebarOpen && (
        <div className="mobile-overlay" onClick={() => setSidebarOpen(false)} />
      )}

      <aside className={`sidebar ${isMobile && sidebarOpen ? 'open' : ''}`}>
        <div className="sidebar-header">
          <h1>CaseHelper</h1>
        </div>

        <div className="case-list">
          {cases.length === 0 ? (
            <div className="panel-empty">No cases yet</div>
          ) : (
            cases.map((c) => (
              <div
                key={c.id}
                className={`case-item ${selectedCaseId === c.id ? 'selected' : ''}`}
                onClick={() => handleCaseSelect(c.id)}
                title={c.name}
              >
                <div className="case-item-name">{c.name}</div>
                <div className="case-item-meta">
                  <span className="stage-tag">{c.stage}</span>
                </div>
              </div>
            ))
          )}
        </div>
      </aside>

      <main className="main-workspace">
        <header className="topbar">
          {isMobile && (
            <button
              className="icon-btn hamburger"
              onClick={() => setSidebarOpen(!sidebarOpen)}
              aria-label="Toggle sidebar"
            >
              <svg width="20" height="20" viewBox="0 0 20 20" fill="none">
                <path d="M3 5h14M3 10h14M3 15h14" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round"/>
              </svg>
            </button>
          )}

          <div className="search-bar">
            <input
              type="text"
              placeholder="Search archive..."
              onKeyDown={(e) => {
                if (e.key === 'Enter') {
                  const query = e.currentTarget.value.trim();
                  if (query) {
                    navigate(`/archive?q=${encodeURIComponent(query)}`);
                  }
                }
              }}
            />
          </div>

          <div className="topbar-actions">
            <button
              className="icon-btn"
              onClick={toggleTodoistPanel}
              aria-label="Toggle Todoist panel"
              title="Todoist"
            >
              <svg width="20" height="20" viewBox="0 0 20 20" fill="none">
                <path d="M4 6h12M4 10h12M4 14h8" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round"/>
              </svg>
            </button>
            <button
              className="icon-btn"
              onClick={() => navigate('/settings')}
              aria-label="Settings"
              title="Settings"
            >
              <svg width="20" height="20" viewBox="0 0 20 20" fill="none">
                <path d="M10 12.5a2.5 2.5 0 100-5 2.5 2.5 0 000 5z" stroke="currentColor" strokeWidth="1.5"/>
                <path d="M16.18 12.5c0-.07-.01-.14-.03-.21l1.82-1.55a.37.37 0 00.07-.47l-1.54-2.67a.37.37 0 00-.44-.15l-2.14.88a3.04 3.04 0 00-.9-.5l-.35-2.43a.37.37 0 00-.37-.32H9.63a.37.37 0 00-.37.32l-.35 2.43c-.33.12-.64.28-.9.5l-2.14-.88a.37.37 0 00-.44.15L2.97 9.9a.37.37 0 00.07.47l1.82 1.55c-.02.07-.03.14-.03.21s.01.14.03.21L2.97 15.3a.37.37 0 00-.07.47l1.54 2.67c.08.14.27.2.44.15l2.14-.88c.26.22.57.38.9.5l.35 2.43c.04.26.3.39.53.28l2.23-1.06c.26-.12.56-.12.82 0l2.23 1.06c.23.11.49-.02.53-.28l.35-2.43c.33-.12.64-.28.9-.5l2.14.88c.17.05.36-.01.44-.15l1.54-2.67a.37.37 0 00-.07-.47l-1.82-1.55c.02-.07.03-.14.03-.21h.03z" stroke="currentColor" strokeWidth="1.5"/>
              </svg>
            </button>
          </div>
        </header>

        <div className="workspace-content">
          {children}
        </div>
      </main>

      {!isMobile && todoistPanelOpen && (
        <aside className="panel">
          <div className="panel-header">
            <span className="panel-title">Todoist</span>
            <button className="icon-btn" onClick={toggleTodoistPanel} aria-label="Collapse panel">
              <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
                <path d="M10 4L6 8l4 4" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round"/>
              </svg>
            </button>
          </div>
          <div className="panel-content">
            <TodoistPanel />
          </div>
        </aside>
      )}
    </div>
  );
}