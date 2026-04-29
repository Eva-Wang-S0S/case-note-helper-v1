import { useEffect } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { useAppStore } from '../stores/appStore';

export function CasePlan() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const caseId = parseInt(id || '0', 10);

  const { cases, selectedCaseId, selectCase, planItems, loadPlanItems } = useAppStore();

  useEffect(() => {
    if (caseId && caseId !== selectedCaseId) {
      selectCase(caseId);
    }
  }, [caseId, selectedCaseId, selectCase]);

  useEffect(() => {
    if (caseId) {
      loadPlanItems(caseId);
    }
  }, [caseId, loadPlanItems]);

  const currentCase = cases.find((c) => c.id === caseId);

  if (!caseId) {
    return (
      <div className="page-container">
        <div className="empty-state">
          <p className="empty-state-description">Select a case to view plan</p>
          <button className="btn btn-secondary" onClick={() => navigate('/')}>
            Back to Dashboard
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="page-container">
      <div className="tab-bar" style={{ marginBottom: '24px' }}>
        <div
          className="tab"
          onClick={() => navigate(`/case/${caseId}/notes`)}
          style={{ cursor: 'pointer' }}
        >
          Notes
        </div>
        <div className="tab active">Plan</div>
      </div>

      <h2 style={{ fontSize: '20px', fontWeight: 600, marginBottom: '24px' }}>
        {currentCase?.name || `Case ${caseId}`}
      </h2>

      {planItems.length === 0 ? (
        <div className="empty-state">
          <h3 className="empty-state-title" style={{ fontSize: '18px' }}>No plan items yet</h3>
          <p className="empty-state-description">
            Plan items will appear here after your first case note is drafted.
            Smart scheduling assistant coming in v1.1.
          </p>
        </div>
      ) : (
        <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
          {planItems.map((item) => (
            <div
              key={item.id}
              style={{
                padding: '16px',
                background: 'var(--color-surface)',
                border: '1px solid var(--color-border)',
                borderRadius: 'var(--radius-md)',
              }}
            >
              <div style={{ display: 'flex', alignItems: 'flex-start', gap: '12px' }}>
                <input
                  type="checkbox"
                  checked={item.completed}
                  onChange={() => {}}
                  style={{ marginTop: '2px' }}
                />
                <div style={{ flex: 1 }}>
                  <p style={{
                    textDecoration: item.completed ? 'line-through' : 'none',
                    color: item.completed ? 'var(--color-text-muted)' : 'var(--color-text-primary)',
                  }}>
                    {item.content}
                  </p>
                  {item.scheduled_date && (
                    <p style={{ fontSize: '12px', color: 'var(--color-text-muted)', marginTop: '4px' }}>
                      Scheduled: {item.scheduled_date}
                    </p>
                  )}
                </div>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}