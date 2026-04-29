import { describe, it, expect } from 'vitest';
import { act } from '@testing-library/react';
import { useAppStore } from './appStore';

describe('appStore', () => {
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
});
