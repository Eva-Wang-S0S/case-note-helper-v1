import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
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
          { id: 7, case_id: 5, content: 'Item A', scheduled_date: null, completed: false, created_at: '1', updated_at: '1' },
          { id: 8, case_id: 5, content: 'Item B', scheduled_date: null, completed: false, created_at: '1', updated_at: '1' },
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
      { id: 1, case_id: 5, content: 'Item 1', scheduled_date: null, completed: false, created_at: '1', updated_at: '1' },
      { id: 2, case_id: 5, content: 'Item 2', scheduled_date: '2026-05-01', completed: true, created_at: '1', updated_at: '1' },
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
});
