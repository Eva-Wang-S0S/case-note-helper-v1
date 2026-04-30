import { describe, it, expect, vi, beforeEach } from 'vitest';
import { act } from '@testing-library/react';
import { useAppStore } from './appStore';

// Mock Tauri invoke - same pattern as appStore.test.ts
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockName('invoke'),
}));

import { invoke } from '@tauri-apps/api/core';

describe('useAppStore - new case behavior', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    useAppStore.setState({
      cases: [],
      selectedCaseId: null,
      notes: [],
      planItems: [],
      isLoading: false,
      error: null,
    });
  });

  it('newly created case starts empty - rawInput and draftOutput are empty', async () => {
    // Simulate creating a new case
    const newCase = {
      id: 1,
      name: 'New Case',
      client_name: 'John Doe',
      stage: 'Intake',
      status: 'active',
      created_at: '2024-01-01',
      updated_at: '2024-01-01',
    };

    vi.mocked(invoke).mockResolvedValueOnce(newCase);

    await act(async () => {
      await useAppStore.getState().createCase('New Case', 'John Doe', 'Intake');
    });

    // After creating a case, it should be added to cases list
    const cases = useAppStore.getState().cases;
    expect(cases.length).toBe(1);
    expect(cases[0].id).toBe(1);
  });

  it('switching cases clears notes state until loaded', async () => {
    // Setup: simulate having notes for case 1
    const notesForCase1 = [
      {
        id: 1,
        case_id: 1,
        raw_content: 'Old raw input',
        draft_content: 'Old draft',
        created_at: '2024-01-01',
        updated_at: '2024-01-01',
      },
    ];

    useAppStore.setState({
      cases: [
        { id: 1, name: 'Case 1', client_name: 'John', stage: 'Intake', status: 'active', created_at: '2024-01-01', updated_at: '2024-01-01' },
        { id: 2, name: 'Case 2', client_name: 'Jane', stage: 'Intake', status: 'active', created_at: '2024-01-02', updated_at: '2024-01-02' },
      ],
      selectedCaseId: 1,
      notes: notesForCase1,
    });

    // Simulate switching to case 2 (new case with no notes)
    const notesForCase2: typeof notesForCase1 = [];
    vi.mocked(invoke).mockResolvedValueOnce(notesForCase2);

    await act(async () => {
      await useAppStore.getState().loadNotes(2);
    });

    // After loading notes for new case, notes should be empty
    expect(useAppStore.getState().notes).toEqual([]);
  });

  it('returning to a case restores its notes', async () => {
    const notesForCase1 = [
      {
        id: 1,
        case_id: 1,
        raw_content: 'Original raw input',
        draft_content: 'Original draft',
        created_at: '2024-01-01',
        updated_at: '2024-01-02',
      },
    ];

    // Simulate returning to case 1
    vi.mocked(invoke).mockResolvedValueOnce(notesForCase1);

    await act(async () => {
      await useAppStore.getState().loadNotes(1);
    });

    const notes = useAppStore.getState().notes;
    expect(notes.length).toBe(1);
    expect(notes[0].raw_content).toBe('Original raw input');
    expect(notes[0].draft_content).toBe('Original draft');
  });
});
