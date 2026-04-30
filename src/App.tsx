import { useEffect } from 'react';
import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import { useAppStore } from './stores/appStore';
import { Layout } from './components/Layout';
import { Dashboard } from './pages/Dashboard';
import { CaseNotes } from './pages/CaseNotes';
import { CasePlan } from './pages/CasePlan';
import { Settings } from './pages/Settings';
import { CaseNoteSettings } from './pages/CaseNoteSettings';
import { Archive } from './pages/Archive';
import './index.css';

export default function App() {
  const loadSettings = useAppStore((s) => s.loadSettings);

  useEffect(() => {
    loadSettings();
  }, [loadSettings]);

  return (
    <BrowserRouter>
      <Layout>
        <Routes>
          <Route path="/" element={<Dashboard />} />
          <Route path="/case/:id/notes" element={<CaseNotes />} />
          <Route path="/case/:id/plan" element={<CasePlan />} />
          <Route path="/settings" element={<Settings />} />
          <Route path="/settings/casenotes" element={<CaseNoteSettings />} />
          <Route path="/archive" element={<Archive />} />
          <Route path="*" element={<Navigate to="/" replace />} />
        </Routes>
      </Layout>
    </BrowserRouter>
  );
}