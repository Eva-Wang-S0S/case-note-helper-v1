import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useAppStore } from '../stores/appStore';
import { TodoistPanel } from './TodoistPanel';

const STAGES = ['Intake', 'Assessment', 'Support Plan', 'Review', 'Closed'];

const STAGE_COLORS: Record<string, string> = {
  'Intake': 'intake',
  'Assessment': 'assessment',
  'Support Plan': 'support-plan',
  'Review': 'review',
  'Closed': 'closed',
};

interface LayoutProps {
  children: React.ReactNode;
}

export function Layout({ children }: LayoutProps) {
  const [sidebarOpen, setSidebarOpen] = useState(false);
  const [showNewCaseModal, setShowNewCaseModal] = useState(false);
  const [newCaseName, setNewCaseName] = useState('');
  const [newClientName, setNewClientName] = useState('');
  const [newStage, setNewStage] = useState('Intake');
  const [activeContextCase, setActiveContextCase] = useState<number | null>(null);
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);
  const [caseToDelete, setCaseToDelete] = useState<{ id: number; name: string } | null>(null);
  const navigate = useNavigate();
  const { cases, selectedCaseId, todoistPanelOpen, toggleTodoistPanel, createCase, updateCaseStage, deleteCase, isLoading } = useAppStore();

  const isMobile = typeof window !== 'undefined' && window.innerWidth < 1024;

  const handleCaseSelect = (caseId: number) => {
    navigate(`/case/${caseId}/notes`);
    setSidebarOpen(false);
  };

  const handleCreateCase = async () => {
    if (!newCaseName.trim() || !newClientName.trim()) return;
    try {
      const newCase = await createCase(newCaseName.trim(), newClientName.trim(), newStage);
      setShowNewCaseModal(false);
      setNewCaseName('');
      setNewClientName('');
      setNewStage('Intake');
      navigate(`/case/${newCase.id}/notes`);
    } catch (e) {
      // error handled in store
    }
  };

  const handleChangeStage = async (caseId: number, stage: string) => {
    try {
      await updateCaseStage(caseId, stage);
    } catch (e) {
      // error handled in store
    }
    setActiveContextCase(null);
  };

  const handleDeleteCase = async () => {
    if (!caseToDelete) return;
    try {
      await deleteCase(caseToDelete.id);
      setShowDeleteConfirm(false);
      setCaseToDelete(null);
      navigate('/');
    } catch (e) {
      // error handled in store
    }
  };

  const confirmDelete = (caseId: number, caseName: string) => {
    setCaseToDelete({ id: caseId, name: caseName });
    setShowDeleteConfirm(true);
    setActiveContextCase(null);
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
                  <div
                    className={`stage-dot stage-dot-${STAGE_COLORS[c.stage] || 'intake'}`}
                    title={c.stage}
                  />
                </div>
                {selectedCaseId === c.id && (
                  <div className="case-item-actions" onClick={(e) => e.stopPropagation()}>
                    <button
                      className="case-context-btn"
                      onClick={() => setActiveContextCase(activeContextCase === c.id ? null : c.id)}
                      title="Case options"
                    >
                      <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
                        <circle cx="7" cy="3" r="1.2" fill="currentColor"/>
                        <circle cx="7" cy="7" r="1.2" fill="currentColor"/>
                        <circle cx="7" cy="11" r="1.2" fill="currentColor"/>
                      </svg>
                    </button>
                    {activeContextCase === c.id && (
                      <div className="case-context-menu">
                        <div className="context-menu-section">
                          <div className="context-menu-label">Change Stage</div>
                          {STAGES.map((stage) => (
                            <button
                              key={stage}
                              className={`context-menu-item ${c.stage === stage ? 'active' : ''}`}
                              onClick={() => handleChangeStage(c.id, stage)}
                            >
                              <span className={`stage-dot stage-dot-${STAGE_COLORS[stage]} ${c.stage === stage ? '' : 'muted'}`} />
                              {stage}
                            </button>
                          ))}
                        </div>
                        <button
                          className="context-menu-item danger"
                          onClick={() => confirmDelete(c.id, c.name)}
                        >
                          Delete Case
                        </button>
                      </div>
                    )}
                  </div>
                )}
              </div>
            ))
          )}
        </div>

        <div className="sidebar-footer">
          <button
            className="btn btn-secondary"
            onClick={() => setShowNewCaseModal(true)}
            style={{ width: '100%', justifyContent: 'center' }}
          >
            + Add Case
          </button>
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

      {showNewCaseModal && (
        <div className="modal-overlay" onClick={() => setShowNewCaseModal(false)}>
          <div className="modal" onClick={(e) => e.stopPropagation()}>
            <h3 className="modal-title">New Case</h3>
            <div className="form-group">
              <label className="form-label">Case Name</label>
              <input
                type="text"
                className="input"
                value={newCaseName}
                onChange={(e) => setNewCaseName(e.target.value)}
                placeholder="e.g., Case #2024-042"
              />
            </div>
            <div className="form-group">
              <label className="form-label">Client Name</label>
              <input
                type="text"
                className="input"
                value={newClientName}
                onChange={(e) => setNewClientName(e.target.value)}
                placeholder="e.g., Jane Smith"
              />
            </div>
            <div className="form-group">
              <label className="form-label">Stage</label>
              <select
                className="input"
                value={newStage}
                onChange={(e) => setNewStage(e.target.value)}
              >
                <option value="Intake">Intake</option>
                <option value="Assessment">Assessment</option>
                <option value="Support Plan">Support Plan</option>
                <option value="Review">Review</option>
                <option value="Closed">Closed</option>
              </select>
            </div>
            <div className="modal-actions">
              <button className="btn btn-secondary" onClick={() => setShowNewCaseModal(false)}>
                Cancel
              </button>
              <button
                className="btn btn-primary"
                onClick={handleCreateCase}
                disabled={!newCaseName.trim() || !newClientName.trim() || isLoading}
              >
                {isLoading ? <span className="btn-spinner" /> : 'Create Case'}
              </button>
            </div>
          </div>
        </div>
      )}

      {showDeleteConfirm && caseToDelete && (
        <div className="modal-overlay" onClick={() => setShowDeleteConfirm(false)}>
          <div className="modal" onClick={(e) => e.stopPropagation()}>
            <h3 className="modal-title">Delete Case</h3>
            <p style={{ fontSize: '14px', color: 'var(--color-text-secondary)', marginBottom: '24px' }}>
              Are you sure you want to permanently delete <strong>{caseToDelete.name}</strong>? This action cannot be undone.
            </p>
            <div className="modal-actions">
              <button className="btn btn-secondary" onClick={() => setShowDeleteConfirm(false)}>
                Cancel
              </button>
              <button
                className="btn btn-danger"
                onClick={handleDeleteCase}
                disabled={isLoading}
              >
                {isLoading ? <span className="btn-spinner" /> : 'Delete Case'}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}