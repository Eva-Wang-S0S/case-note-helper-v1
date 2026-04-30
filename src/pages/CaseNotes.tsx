import { useState, useEffect, useCallback, useRef } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { useAppStore } from '../stores/appStore';

export function CaseNotes() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const caseId = parseInt(id || '0', 10);

  const {
    selectedCaseId,
    selectCase,
    saveNote,
    draftNote,
    error,
    setError,
    notes,
  } = useAppStore();

  const [rawInput, setRawInput] = useState('');
  const [draftOutput, setDraftOutput] = useState('');
  const [anonymize, setAnonymize] = useState(false);
  const [isDrafting, setIsDrafting] = useState(false);
  const [draftError, setDraftError] = useState<string | null>(null);
  const [showSaved, setShowSaved] = useState(false);
  const [validationError, setValidationError] = useState<string | null>(null);

  const autoSaveTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const saveIndicatorTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const initializedRef = useRef(false);

  // Load existing notes when case is selected
  useEffect(() => {
    // Reset initialization flag and clear content when switching cases
    initializedRef.current = false;
    setRawInput('');
    setDraftOutput('');
    if (caseId && caseId !== selectedCaseId) {
      selectCase(caseId);
    }
  }, [caseId, selectedCaseId, selectCase]);

  // Populate rawInput and draftOutput from loaded notes
  useEffect(() => {
    if (notes.length > 0 && !initializedRef.current) {
      // Sort by updated_at descending to get the most recent note
      const sortedNotes = [...notes].sort(
        (a, b) => new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime()
      );
      const latestNote = sortedNotes[0];
      setRawInput(latestNote.raw_content || '');
      setDraftOutput(latestNote.draft_content || '');
      initializedRef.current = true;
    }
  }, [notes]);

  const triggerSave = useCallback(async () => {
    if (!caseId || !rawInput.trim()) return;
    try {
      await saveNote(caseId, rawInput, draftOutput || undefined);
      setShowSaved(true);
      if (saveIndicatorTimer.current) {
        clearTimeout(saveIndicatorTimer.current);
      }
      saveIndicatorTimer.current = setTimeout(() => setShowSaved(false), 2000);
    } catch (e) {
      setError(String(e));
    }
  }, [caseId, rawInput, draftOutput, saveNote, setError]);

  // Save handler for manual save button
  const handleSaveDraft = async () => {
    if (!caseId || !rawInput.trim()) return;
    setValidationError(null);
    try {
      await saveNote(caseId, rawInput, draftOutput || undefined);
      setShowSaved(true);
      if (saveIndicatorTimer.current) {
        clearTimeout(saveIndicatorTimer.current);
      }
      saveIndicatorTimer.current = setTimeout(() => setShowSaved(false), 2000);
    } catch (e) {
      setError(String(e));
    }
  };

  useEffect(() => {
    if (autoSaveTimer.current) {
      clearTimeout(autoSaveTimer.current);
    }
    if (rawInput.trim()) {
      autoSaveTimer.current = setTimeout(() => {
        triggerSave();
      }, 30000);
    }
    return () => {
      if (autoSaveTimer.current) {
        clearTimeout(autoSaveTimer.current);
      }
    };
  }, [rawInput, triggerSave]);

  const handleDraft = async () => {
    const trimmed = rawInput.trim();
    if (!trimmed) {
      setValidationError('Please enter your observations before drafting.');
      return;
    }
    setValidationError(null);
    setDraftError(null);
    setIsDrafting(true);

    try {
      const draft = await draftNote(trimmed, anonymize);
      setDraftOutput(draft);
      // Save raw input alongside draft result
      await saveNote(caseId, rawInput, draft);
    } catch (e) {
      setDraftError('Draft failed. Check your connection or try again.');
      // Save raw input even on error so it's not lost
      try {
        await saveNote(caseId, rawInput, draftOutput || undefined);
      } catch {
        // ignore save error here, main error is the draft failure
      }
    } finally {
      setIsDrafting(false);
    }
  };

  const handleCopyDraft = () => {
    if (draftOutput) {
      navigator.clipboard.writeText(draftOutput);
    }
  };

  if (!caseId) {
    return (
      <div className="page-container">
        <div className="empty-state">
          <p className="empty-state-description">Select a case to view notes</p>
          <button className="btn btn-secondary" onClick={() => navigate('/')}>
            Back to Dashboard
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="page-container" style={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
      <div className="tab-bar" style={{ marginBottom: '24px', display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
        <div style={{ display: 'flex', gap: '4px' }}>
          <div
            className="tab active"
            onClick={() => {}}
          >
            Notes
          </div>
          <div
            className="tab"
            onClick={() => navigate(`/case/${caseId}/plan`)}
            style={{ cursor: 'pointer' }}
          >
            Plan
          </div>
        </div>
        <button
          className="btn btn-ghost"
          onClick={() => navigate('/settings/casenotes')}
          style={{ padding: '6px', display: 'flex', alignItems: 'center', gap: '6px', fontSize: '13px' }}
          title="Case note settings"
        >
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
            <path d="M8 10.5a2.5 2.5 0 100-5 2.5 2.5 0 000 5z" stroke="currentColor" strokeWidth="1.2"/>
            <path d="M6.5 1.5h3M8 1.5v1.5M11.5 4l-.8.6M12.5 7h1.5M11.5 10l-.8-.6M8 13.5V12M4.5 4l.8.6M3.5 7H2M4.5 10l.8-.6" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round"/>
          </svg>
          Settings
        </button>
      </div>

      {error && (
        <div className="error-banner">
          <span className="error-banner-message">{error}</span>
          <button className="btn btn-ghost" onClick={() => setError(null)}>Dismiss</button>
        </div>
      )}

      <div className="case-notes-layout" style={{ flex: 1, minHeight: 0 }}>
        <div className="case-notes-input">
          <div className="notes-topbar">
            <h3 style={{ fontSize: '16px', fontWeight: 600 }}>Raw Observations</h3>
          </div>

          <div className="form-group" style={{ flex: 1, display: 'flex', flexDirection: 'column' }}>
            <textarea
              className={`textarea ${isDrafting ? 'dimmed' : ''}`}
              value={rawInput}
              onChange={(e) => {
                setRawInput(e.target.value);
                setValidationError(null);
              }}
              placeholder="Enter your observations, notes, and any relevant details..."
              style={{ flex: 1 }}
            />
          </div>

          {validationError && (
            <div style={{ color: 'var(--color-error)', fontSize: '14px', marginTop: '8px' }}>
              {validationError}
            </div>
          )}

          <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: '16px' }}>
              <label
                className="toggle"
                onClick={() => setAnonymize(!anonymize)}
              >
                <div className={`toggle-track ${anonymize ? 'on' : ''}`}>
                  <div className="toggle-knob" />
                </div>
                <span className="toggle-label">Anonymize</span>
              </label>
            </div>

            <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
              {showSaved && <span className="saved-indicator">Saved</span>}
              <button
                className="btn btn-primary"
                onClick={handleDraft}
                disabled={isDrafting || !rawInput.trim()}
              >
                {isDrafting ? (
                  <>
                    <span className="btn-spinner" />
                    Drafting...
                  </>
                ) : (
                  'Draft'
                )}
              </button>
            </div>
          </div>
        </div>

        <div className="case-notes-preview">
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
            <h3 style={{ fontSize: '16px', fontWeight: 600 }}>Draft</h3>
            {draftOutput && (
              <div style={{ display: 'flex', gap: '8px' }}>
                <button className="btn btn-secondary" onClick={handleSaveDraft}>
                  Save
                </button>
                <button className="btn btn-secondary" onClick={handleCopyDraft}>
                  Copy
                </button>
              </div>
            )}
          </div>

          {isDrafting ? (
            <div className="drafting-placeholder">Drafting...</div>
          ) : draftError ? (
            <div className="error-banner" style={{ marginTop: '16px' }}>
              <span className="error-banner-message">{draftError}</span>
              <button className="btn btn-ghost" onClick={handleDraft}>Retry</button>
            </div>
          ) : draftOutput ? (
            <textarea
              className="textarea draft-textarea"
              value={draftOutput}
              onChange={(e) => setDraftOutput(e.target.value)}
              style={{ flex: 1 }}
            />
          ) : (
            <div
              style={{
                flex: 1,
                border: '1px dashed var(--color-border)',
                borderRadius: 'var(--radius-md)',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                color: 'var(--color-text-muted)',
                fontSize: '14px',
              }}
            >
              Draft will appear here
            </div>
          )}
        </div>
      </div>
    </div>
  );
}