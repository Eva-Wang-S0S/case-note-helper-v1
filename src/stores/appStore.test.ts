import { describe, it, expect, vi, beforeEach } from 'vitest';
import { act } from '@testing-library/react';
import { useAppStore } from './appStore';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockName('invoke'),
}));

import { invoke } from '@tauri-apps/api/core';

describe('appStore', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    useAppStore.setState({
      planItems: [],
      error: null,
      cases: [],
      selectedCaseId: null,
      notes: [],
      isLoading: false,
    });
  });

  it('has correct initial state', () => {
    const state = useAppStore.getState();
    expect(state.cases).toEqual([]);
    expect(state.selectedCaseId).toBeNull();
    expect(state.notes).toEqual([]);
    expect(state.planItems).toEqual([]);
    expect(state.settings.llm_provider).toBe('ollama');
    expect(state.todoistPanelOpen).toBe(true);
    expect(state.error).toBeNull();
  });

  it('can set and clear error', () => {
    const { setError } = useAppStore.getState();
    act(() => setError('test error'));
    expect(useAppStore.getState().error).toBe('test error');
    act(() => setError(null));
    expect(useAppStore.getState().error).toBeNull();
  });

  it('can toggle todoist panel', () => {
    const { toggleTodoistPanel } = useAppStore.getState();
    expect(useAppStore.getState().todoistPanelOpen).toBe(true);
    act(() => toggleTodoistPanel());
    expect(useAppStore.getState().todoistPanelOpen).toBe(false);
    act(() => toggleTodoistPanel());
    expect(useAppStore.getState().todoistPanelOpen).toBe(true);
  });

  describe('createPlanItem', () => {
    const mockItem = {
      id: 1,
      case_id: 5,
      content: 'Follow up with client',
      scheduled_date: null,
      completed: false,
      created_at: '1234567890',
      updated_at: '1234567890',
      todoist_task_id: null,
      sync_status: 'synced',
      last_synced_at: null,
    };

    it('adds plan item to state on success', async () => {
      vi.mocked(invoke).mockResolvedValueOnce(mockItem);
      const { createPlanItem } = useAppStore.getState();

      await act(async () => {
        await createPlanItem(5, 'Follow up with client');
      });

      expect(invoke).toHaveBeenCalledWith('create_plan_item', {
        caseId: 5,
        content: 'Follow up with client',
        scheduledDate: null,
      });
      expect(useAppStore.getState().planItems).toContainEqual(mockItem);
    });

    it('sets error on failure', async () => {
      vi.mocked(invoke).mockRejectedValueOnce(new Error('DB error'));
      const { createPlanItem } = useAppStore.getState();

      await act(async () => {
        try {
          await createPlanItem(5, 'Follow up');
        } catch {}
      });

      expect(useAppStore.getState().error).toMatch('DB error');
    });
  });

  describe('togglePlanItem', () => {
    const mockUpdated = {
      id: 3,
      case_id: 5,
      content: 'Existing item',
      scheduled_date: null,
      completed: true,
      created_at: '1234567890',
      updated_at: '1234567891',
      todoist_task_id: null,
      sync_status: 'synced',
      last_synced_at: null,
    };

    it('updates plan item in state on success', async () => {
      vi.mocked(invoke).mockResolvedValueOnce(mockUpdated);
      useAppStore.setState({
        planItems: [{ ...mockUpdated, completed: false }],
      });
      const { togglePlanItem } = useAppStore.getState();

      await act(async () => {
        await togglePlanItem(3);
      });

      expect(invoke).toHaveBeenCalledWith('toggle_plan_item', { itemId: 3 });
      expect(useAppStore.getState().planItems.find((i) => i.id === 3)?.completed).toBe(true);
    });

    it('sets error on failure', async () => {
      vi.mocked(invoke).mockRejectedValueOnce(new Error('toggle failed'));
      const { togglePlanItem } = useAppStore.getState();

      await act(async () => {
        try {
          await togglePlanItem(99);
        } catch {}
      });

      expect(useAppStore.getState().error).toMatch('toggle failed');
    });
  });

  describe('deletePlanItem', () => {
    it('removes plan item from state on success', async () => {
      vi.mocked(invoke).mockResolvedValueOnce(undefined);
      useAppStore.setState({
        planItems: [
          { id: 7, case_id: 5, content: 'Item A', scheduled_date: null, completed: false, created_at: '1', updated_at: '1', todoist_task_id: null, sync_status: 'synced', last_synced_at: null },
          { id: 8, case_id: 5, content: 'Item B', scheduled_date: null, completed: false, created_at: '1', updated_at: '1', todoist_task_id: null, sync_status: 'synced', last_synced_at: null },
        ],
      });
      const { deletePlanItem } = useAppStore.getState();

      await act(async () => {
        await deletePlanItem(7);
      });

      expect(invoke).toHaveBeenCalledWith('delete_plan_item', { itemId: 7 });
      const remaining = useAppStore.getState().planItems;
      expect(remaining.map((i) => i.id)).not.toContain(7);
      expect(remaining.map((i) => i.id)).toContain(8);
    });

    it('sets error on failure', async () => {
      vi.mocked(invoke).mockRejectedValueOnce(new Error('delete failed'));
      const { deletePlanItem } = useAppStore.getState();

      await act(async () => {
        try {
          await deletePlanItem(99);
        } catch {}
      });

      expect(useAppStore.getState().error).toMatch('delete failed');
    });
  });

  describe('loadPlanItems', () => {
    const mockItems = [
      { id: 1, case_id: 5, content: 'Item 1', scheduled_date: null, completed: false, created_at: '1', updated_at: '1', todoist_task_id: null, sync_status: 'synced', last_synced_at: null },
      { id: 2, case_id: 5, content: 'Item 2', scheduled_date: '2026-05-01', completed: true, created_at: '1', updated_at: '1', todoist_task_id: null, sync_status: 'synced', last_synced_at: null },
    ];

    it('loads plan items into state', async () => {
      vi.mocked(invoke).mockResolvedValueOnce(mockItems);
      const { loadPlanItems } = useAppStore.getState();

      await act(async () => {
        await loadPlanItems(5);
      });

      expect(invoke).toHaveBeenCalledWith('get_plan_items', { caseId: 5 });
      expect(useAppStore.getState().planItems).toEqual(mockItems);
    });

    it('sets error on failure', async () => {
      vi.mocked(invoke).mockRejectedValueOnce(new Error('load failed'));
      const { loadPlanItems } = useAppStore.getState();

      await act(async () => {
        await loadPlanItems(5);
      });

      expect(useAppStore.getState().error).toMatch('load failed');
    });
  });

  describe('updateCaseStage', () => {
    const mockCase = {
      id: 1,
      name: 'Test Case',
      client_name: 'John Doe',
      stage: 'Intake',
      status: 'active',
      created_at: '2024-01-01',
      updated_at: '2024-01-01',
    };

    it('updates case stage in state on success', async () => {
      const updatedCase = { ...mockCase, stage: 'Assessment' };
      vi.mocked(invoke).mockResolvedValueOnce(updatedCase);
      useAppStore.setState({ cases: [mockCase] });
      const { updateCaseStage } = useAppStore.getState();

      await act(async () => {
        await updateCaseStage(1, 'Assessment');
      });

      expect(invoke).toHaveBeenCalledWith('update_case_stage', { caseId: 1, stage: 'Assessment' });
      expect(useAppStore.getState().cases.find((c) => c.id === 1)?.stage).toBe('Assessment');
    });

    it('sets error on failure', async () => {
      vi.mocked(invoke).mockRejectedValueOnce(new Error('update failed'));
      const { updateCaseStage } = useAppStore.getState();

      await act(async () => {
        try {
          await updateCaseStage(1, 'Review');
        } catch {}
      });

      expect(useAppStore.getState().error).toMatch('update failed');
    });
  });

  describe('deleteCase', () => {
    const mockCases = [
      { id: 1, name: 'Case 1', client_name: 'John', stage: 'Intake', status: 'active', created_at: '2024-01-01', updated_at: '2024-01-01' },
      { id: 2, name: 'Case 2', client_name: 'Jane', stage: 'Intake', status: 'active', created_at: '2024-01-02', updated_at: '2024-01-02' },
    ];

    it('removes case from state on success', async () => {
      vi.mocked(invoke).mockResolvedValueOnce(undefined);
      useAppStore.setState({ cases: [...mockCases] });
      const { deleteCase } = useAppStore.getState();

      await act(async () => {
        await deleteCase(1);
      });

      expect(invoke).toHaveBeenCalledWith('delete_case', { caseId: 1 });
      const remaining = useAppStore.getState().cases;
      expect(remaining.map((c) => c.id)).not.toContain(1);
      expect(remaining.map((c) => c.id)).toContain(2);
    });

    it('clears selectedCaseId if deleted case was selected', async () => {
      vi.mocked(invoke).mockResolvedValueOnce(undefined);
      useAppStore.setState({ cases: [mockCases[0]], selectedCaseId: 1 });
      const { deleteCase } = useAppStore.getState();

      await act(async () => {
        await deleteCase(1);
      });

      expect(useAppStore.getState().selectedCaseId).toBeNull();
    });

    it('does not clear selectedCaseId if deleted case was not selected', async () => {
      vi.mocked(invoke).mockResolvedValueOnce(undefined);
      useAppStore.setState({ cases: [...mockCases], selectedCaseId: 2 });
      const { deleteCase } = useAppStore.getState();

      await act(async () => {
        await deleteCase(1);
      });

      expect(useAppStore.getState().selectedCaseId).toBe(2);
    });

    it('sets error on failure', async () => {
      vi.mocked(invoke).mockRejectedValueOnce(new Error('delete failed'));
      const { deleteCase } = useAppStore.getState();

      await act(async () => {
        try {
          await deleteCase(99);
        } catch {}
      });

      expect(useAppStore.getState().error).toMatch('delete failed');
    });
  });

  describe('settings', () => {
    it('has llm_note_prompt in initial settings', () => {
      const state = useAppStore.getState();
      expect(state.settings.llm_note_prompt).toBeTruthy();
      expect(typeof state.settings.llm_note_prompt).toBe('string');
    });

    it('saveSettings includes llm_note_prompt', async () => {
      vi.mocked(invoke).mockResolvedValueOnce(undefined);
      const { saveSettings } = useAppStore.getState();

      const settings = {
        llm_provider: 'ollama',
        llm_endpoint: 'http://localhost:11434/v1/chat/completions',
        llm_api_key: '',
        llm_model: 'llama3.2',
        redaction_list: ['Test'],
        llm_note_prompt: 'Custom prompt',
      };

      await act(async () => {
        await saveSettings(settings);
      });

      expect(invoke).toHaveBeenCalledWith('save_settings', { settings });
    });

    it('loadSettings loads llm_note_prompt from backend', async () => {
      const mockSettings = {
        llm_provider: 'openai',
        llm_endpoint: 'https://api.openai.com/v1/chat/completions',
        llm_api_key: 'sk-test',
        llm_model: 'gpt-4o-mini',
        redaction_list: ['Name'],
        llm_note_prompt: 'My custom prompt',
      };
      vi.mocked(invoke).mockResolvedValueOnce(mockSettings);
      const { loadSettings } = useAppStore.getState();

      await act(async () => {
        await loadSettings();
      });

      const settings = useAppStore.getState().settings;
      expect(settings.llm_note_prompt).toBe('My custom prompt');
      expect(settings.llm_provider).toBe('openai');
    });
  });
});
