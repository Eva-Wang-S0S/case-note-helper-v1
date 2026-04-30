import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useAppStore } from '../stores/appStore';

export function CaseNoteSettings() {
  const navigate = useNavigate();
  const { settings, saveSettings, isLoading, error, setError } = useAppStore();

  const [form, setForm] = useState({
    redaction_list: [...settings.redaction_list],
    llm_note_prompt: settings.llm_note_prompt,
  });
  const [showSuccess, setShowSuccess] = useState(false);

  const handleSave = async () => {
    try {
      await saveSettings({
        ...settings,
        redaction_list: form.redaction_list,
        llm_note_prompt: form.llm_note_prompt,
      });
      setShowSuccess(true);
      setTimeout(() => setShowSuccess(false), 3000);
    } catch (e) {
      setError(String(e));
    }
  };

  const addRedactionTerm = () => {
    setForm({
      ...form,
      redaction_list: [...form.redaction_list, ''],
    });
  };

  const updateRedactionTerm = (index: number, value: string) => {
    const newList = [...form.redaction_list];
    newList[index] = value;
    setForm({ ...form, redaction_list: newList });
  };

  const removeRedactionTerm = (index: number) => {
    const newList = form.redaction_list.filter((_, i) => i !== index);
    setForm({ ...form, redaction_list: newList });
  };

  return (
    <div className="page-container">
      <div style={{ display: 'flex', alignItems: 'center', gap: '16px', marginBottom: '32px' }}>
        <button
          className="btn btn-ghost"
          onClick={() => navigate(-1)}
          style={{ padding: '8px' }}
        >
          <svg width="20" height="20" viewBox="0 0 20 20" fill="none">
            <path d="M12 4L6 10l6 6" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round"/>
          </svg>
        </button>
        <h2 style={{ fontSize: '24px', fontWeight: 600 }}>Case Note Settings</h2>
      </div>

      {error && (
        <div className="error-banner">
          <span className="error-banner-message">{error}</span>
          <button className="btn btn-ghost" onClick={() => setError(null)}>Dismiss</button>
        </div>
      )}

      {showSuccess && (
        <div className="success-banner">Settings saved</div>
      )}

      <div style={{ maxWidth: '600px' }}>
        <section style={{ marginBottom: '32px' }}>
          <h3 style={{ fontSize: '16px', fontWeight: 600, marginBottom: '16px' }}>LLM Note Prompt</h3>
          <p style={{ fontSize: '13px', color: 'var(--color-text-secondary)', marginBottom: '16px' }}>
            Customize the prompt sent to the LLM when drafting case notes. Use {"{raw_input}"} as a placeholder for the observations.
          </p>
          <div className="form-group">
            <textarea
              className="textarea"
              value={form.llm_note_prompt}
              onChange={(e) => setForm({ ...form, llm_note_prompt: e.target.value })}
              rows={6}
              style={{ fontSize: '13px' }}
            />
          </div>
        </section>

        <section style={{ marginBottom: '32px' }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '16px' }}>
            <h3 style={{ fontSize: '16px', fontWeight: 600 }}>Redaction List</h3>
            <button className="btn btn-secondary" onClick={addRedactionTerm} style={{ fontSize: '13px', padding: '6px 12px' }}>
              + Add Term
            </button>
          </div>

          <p style={{ fontSize: '13px', color: 'var(--color-text-secondary)', marginBottom: '16px' }}>
            Terms to redact before sending to LLM. Applied when "Anonymize" is enabled. Matching is case-insensitive and applies across the entire text. For example, adding <code style={{ fontSize: '12px', background: 'var(--color-bg)', padding: '2px 4px', borderRadius: '4px' }}>"Smith"</code> will replace all instances of "smith", "Smith", "SMITH" with <code style={{ fontSize: '12px', background: 'var(--color-bg)', padding: '2px 4px', borderRadius: '4px' }}>[REDACTED]</code>.
          </p>
          <div style={{ background: 'var(--color-sidebar)', borderRadius: '6px', padding: '12px', marginBottom: '16px' }}>
            <p style={{ fontSize: '13px', color: 'var(--color-text-secondary)', marginBottom: '8px' }}>
              <strong>Common examples to add:</strong>
            </p>
            <ul style={{ fontSize: '13px', color: 'var(--color-text-secondary)', margin: 0, paddingLeft: '20px' }}>
              <li>Client/guardian names (e.g., <code style={{ fontSize: '12px', background: 'var(--color-surface)', padding: '2px 4px', borderRadius: '4px' }}>John Smith</code>)</li>
              <li>Organizations (e.g., <code style={{ fontSize: '12px', background: 'var(--color-surface)', padding: '2px 4px', borderRadius: '4px' }}>Acme Corp</code>)</li>
              <li>School names, addresses, specific locations</li>
              <li>Case file numbers or reference IDs</li>
            </ul>
          </div>

          <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
            {form.redaction_list.map((term, index) => (
              <div key={index} style={{ display: 'flex', gap: '8px', alignItems: 'center' }}>
                <input
                  type="text"
                  className="input"
                  value={term}
                  onChange={(e) => updateRedactionTerm(index, e.target.value)}
                  placeholder="Term to redact"
                  style={{ flex: 1 }}
                />
                <button
                  className="btn btn-ghost"
                  onClick={() => removeRedactionTerm(index)}
                  style={{ color: 'var(--color-error)', padding: '8px' }}
                >
                  <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
                    <path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round"/>
                  </svg>
                </button>
              </div>
            ))}
          </div>

          {form.redaction_list.length === 0 && (
            <p style={{ fontSize: '13px', color: 'var(--color-text-muted)', fontStyle: 'italic' }}>
              No redaction terms configured. Add terms above.
            </p>
          )}
        </section>

        <div style={{ display: 'flex', gap: '12px' }}>
          <button
            className="btn btn-primary"
            onClick={handleSave}
            disabled={isLoading}
          >
            {isLoading ? <span className="btn-spinner" /> : 'Save Settings'}
          </button>
          <button
            className="btn btn-secondary"
            onClick={() => navigate(-1)}
          >
            Cancel
          </button>
        </div>
      </div>
    </div>
  );
}
