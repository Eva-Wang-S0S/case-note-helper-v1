import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useAppStore } from '../stores/appStore';

export function Dashboard() {
  const { cases, loadCases, createCase, isLoading, error, setError } = useAppStore();
  const navigate = useNavigate();
  const [showModal, setShowModal] = useState(false);
  const [newCaseName, setNewCaseName] = useState('');
  const [newClientName, setNewClientName] = useState('');
  const [newStage, setNewStage] = useState('Intake');

  useEffect(() => {
    loadCases();
  }, [loadCases]);

  const handleCreateCase = async () => {
    if (!newCaseName.trim() || !newClientName.trim()) return;

    try {
      const newCase = await createCase(newCaseName.trim(), newClientName.trim(), newStage);
      setShowModal(false);
      setNewCaseName('');
      setNewClientName('');
      setNewStage('Intake');
      navigate(`/case/${newCase.id}/notes`);
    } catch (e) {
      // error handled in store
    }
  };

  const handleCaseClick = (caseId: number) => {
    navigate(`/case/${caseId}/notes`);
  };

  if (cases.length === 0) {
    return (
      <div className="page-container">
        <div className="empty-state">
          <h2 className="empty-state-title">Welcome to CaseHelper</h2>
          <p className="empty-state-description">
            Add your first case to get started with LLM-assisted case notes.
          </p>
          <button className="btn btn-primary" onClick={() => setShowModal(true)}>
            Add first case
          </button>
        </div>

        {showModal && (
          <div className="modal-overlay" onClick={() => setShowModal(false)}>
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
                <button className="btn btn-secondary" onClick={() => setShowModal(false)}>
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
      </div>
    );
  }

  return (
    <div className="page-container">
      {error && (
        <div className="error-banner">
          <span className="error-banner-message">{error}</span>
          <button className="btn btn-ghost" onClick={() => setError(null)}>Dismiss</button>
        </div>
      )}

      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '24px' }}>
        <h2 style={{ fontSize: '24px', fontWeight: 600 }}>Cases</h2>
        <button className="btn btn-primary" onClick={() => setShowModal(true)}>
          + Add Case
        </button>
      </div>

      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(280px, 1fr))', gap: '16px' }}>
        {cases.map((c) => (
          <div
            key={c.id}
            className="case-item"
            onClick={() => handleCaseClick(c.id)}
            style={{ background: 'var(--color-surface)', border: '1px solid var(--color-border)', borderRadius: 'var(--radius-md)' }}
          >
            <div className="case-item-name">{c.name}</div>
            <div style={{ fontSize: '13px', color: 'var(--color-text-secondary)', marginTop: '4px' }}>
              {c.client_name}
            </div>
            <div className="case-item-meta">
              <span className="stage-tag">{c.stage}</span>
              <span style={{ fontSize: '12px', color: 'var(--color-text-muted)' }}>
                {c.status}
              </span>
            </div>
          </div>
        ))}
      </div>

      {showModal && (
        <div className="modal-overlay" onClick={() => setShowModal(false)}>
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
              <button className="btn btn-secondary" onClick={() => setShowModal(false)}>
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
    </div>
  );
}