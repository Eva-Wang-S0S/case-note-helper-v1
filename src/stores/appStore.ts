import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';

export interface Case {
  id: number;
  name: string;
  client_name: string;
  stage: string;
  status: string;
  created_at: string;
  updated_at: string;
}

export interface Note {
  id: number;
  case_id: number;
  raw_content: string;
  draft_content: string | null;
  created_at: string;
  updated_at: string;
}

export interface PlanItem {
  id: number;
  case_id: number;
  content: string;
  scheduled_date: string | null;
  completed: boolean;
  created_at: string;
  updated_at: string;
  todoist_task_id: string | null;
  sync_status: string;
  last_synced_at: string | null;
}

export interface AppSettings {
  llm_provider: string;
  llm_endpoint: string;
  llm_api_key: string;
  llm_model: string;
  redaction_list: string[];
}

export interface TodoistTask {
  id: string;
  content: string;
  description: string;
  completed: boolean;
  due_date: string | null;
  due_datetime: string | null;
  priority: number;
  project_id: string;
}

export interface TodoistConnectionStatus {
  connected: boolean;
  last_synced_at: string | null;
  rate_limited: boolean;
  rate_limit_until: string | null;
}

interface AppState {
  cases: Case[];
  selectedCaseId: number | null;
  notes: Note[];
  planItems: PlanItem[];
  settings: AppSettings;
  todoistPanelOpen: boolean;
  todoistTasks: TodoistTask[];
  todoistConnectionStatus: TodoistConnectionStatus;
  isLoading: boolean;
  error: string | null;

  loadCases: () => Promise<void>;
  selectCase: (caseId: number | null) => void;
  createCase: (name: string, clientName: string, stage: string) => Promise<Case>;
  loadNotes: (caseId: number) => Promise<void>;
  saveNote: (caseId: number, rawContent: string, draftContent?: string) => Promise<void>;
  draftNote: (rawInput: string, anonymize: boolean) => Promise<string>;
  loadPlanItems: (caseId: number) => Promise<void>;
  createPlanItem: (caseId: number, content: string, scheduledDate?: string) => Promise<PlanItem>;
  togglePlanItem: (itemId: number) => Promise<void>;
  deletePlanItem: (itemId: number) => Promise<void>;
  syncPlanItem: (itemId: number) => Promise<void>;
  loadSettings: () => Promise<void>;
  saveSettings: (settings: AppSettings) => Promise<void>;
  searchArchive: (query: string) => Promise<Case[]>;
  loadTodoistTasks: () => Promise<void>;
  getTodoistConnectionStatus: () => Promise<void>;
  toggleTodoistPanel: () => void;
  setError: (error: string | null) => void;
}

export const useAppStore = create<AppState>((set, get) => ({
  cases: [],
  selectedCaseId: null,
  notes: [],
  planItems: [],
  settings: {
    llm_provider: 'ollama',
    llm_endpoint: 'http://localhost:11434/v1/chat/completions',
    llm_api_key: '',
    llm_model: 'llama3.2',
    redaction_list: ['[REDACTED NAME]', '[REDACTED ORG]', '[REDACTED LOCATION]'],
  },
  todoistPanelOpen: true,
  todoistTasks: [],
  todoistConnectionStatus: { connected: false, last_synced_at: null, rate_limited: false, rate_limit_until: null },
  isLoading: false,
  error: null,

  loadCases: async () => {
    set({ isLoading: true, error: null });
    try {
      const cases = await invoke<Case[]>('get_cases');
      set({ cases, isLoading: false });
    } catch (e) {
      set({ error: String(e), isLoading: false });
    }
  },

  selectCase: (caseId) => {
    set({ selectedCaseId: caseId });
    if (caseId !== null) {
      get().loadNotes(caseId);
      get().loadPlanItems(caseId);
    }
  },

  createCase: async (name, clientName, stage) => {
    set({ isLoading: true, error: null });
    try {
      const newCase = await invoke<Case>('create_case', { name, clientName, stage });
      set((state) => ({
        cases: [...state.cases, newCase],
        isLoading: false,
      }));
      return newCase;
    } catch (e) {
      set({ error: String(e), isLoading: false });
      throw e;
    }
  },

  loadNotes: async (caseId) => {
    try {
      const notes = await invoke<Note[]>('get_notes', { caseId });
      set({ notes });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  saveNote: async (caseId, rawContent, draftContent) => {
    try {
      const note = await invoke<Note>('save_note', { caseId, rawContent, draftContent });
      set((state) => ({
        notes: state.notes.some((n) => n.id === note.id)
          ? state.notes.map((n) => (n.id === note.id ? note : n))
          : [...state.notes, note],
      }));
    } catch (e) {
      set({ error: String(e) });
    }
  },

  draftNote: async (rawInput, anonymize) => {
    set({ isLoading: true, error: null });
    try {
      const draft = await invoke<string>('draft_case_note', { rawInput, anonymize });
      set({ isLoading: false });
      return draft;
    } catch (e) {
      set({ error: String(e), isLoading: false });
      throw e;
    }
  },

  loadPlanItems: async (caseId) => {
    try {
      const planItems = await invoke<PlanItem[]>('get_plan_items', { caseId });
      set({ planItems });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  createPlanItem: async (caseId, content, scheduledDate) => {
    try {
      const item = await invoke<PlanItem>('create_plan_item', {
        caseId,
        content,
        scheduledDate: scheduledDate ?? null,
      });
      set((state) => ({ planItems: [...state.planItems, item] }));
      return item;
    } catch (e) {
      set({ error: String(e) });
      throw e;
    }
  },

  togglePlanItem: async (itemId) => {
    try {
      const updated = await invoke<PlanItem>('toggle_plan_item', { itemId });
      set((state) => ({
        planItems: state.planItems.map((i) => (i.id === itemId ? updated : i)),
      }));
    } catch (e) {
      set({ error: String(e) });
      throw e;
    }
  },

  deletePlanItem: async (itemId) => {
    try {
      await invoke('delete_plan_item', { itemId });
      set((state) => ({
        planItems: state.planItems.filter((i) => i.id !== itemId),
      }));
    } catch (e) {
      set({ error: String(e) });
      throw e;
    }
  },

  syncPlanItem: async (itemId) => {
    try {
      const updated = await invoke<PlanItem>('sync_plan_item_toggle', { itemId });
      set((state) => ({
        planItems: state.planItems.map((i) => (i.id === itemId ? updated : i)),
      }));
    } catch (e) {
      set({ error: String(e) });
      throw e;
    }
  },

  loadSettings: async () => {
    try {
      const settings = await invoke<AppSettings>('get_settings');
      set({ settings });
    } catch (e) {
      console.error('Failed to load settings:', e);
    }
  },

  saveSettings: async (settings) => {
    set({ isLoading: true, error: null });
    try {
      await invoke('save_settings', { settings });
      set({ settings, isLoading: false });
    } catch (e) {
      set({ error: String(e), isLoading: false });
      throw e;
    }
  },

  searchArchive: async (query) => {
    set({ isLoading: true, error: null });
    try {
      const results = await invoke<Case[]>('search_archive', { query });
      set({ isLoading: false });
      return results;
    } catch (e) {
      set({ error: String(e), isLoading: false });
      throw e;
    }
  },

  loadTodoistTasks: async () => {
    try {
      const tasks = await invoke<TodoistTask[]>('get_todoist_tasks');
      set({ todoistTasks: tasks });
    } catch (e) {
      console.error('Failed to load Todoist tasks:', e);
    }
  },

  getTodoistConnectionStatus: async () => {
    try {
      const status = await invoke<TodoistConnectionStatus>('get_todoist_connection_status');
      set({ todoistConnectionStatus: status });
    } catch (e) {
      console.error('Failed to get Todoist connection status:', e);
    }
  },

  toggleTodoistPanel: () => {
    set((state) => ({ todoistPanelOpen: !state.todoistPanelOpen }));
  },

  setError: (error) => set({ error }),
}));
