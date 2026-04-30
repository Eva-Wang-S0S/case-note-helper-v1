import { useEffect } from 'react';
import { useAppStore } from '../stores/appStore';

export function TodoistPanel() {
  const {
    todoistTasks,
    todoistConnectionStatus,
    planItems,
    loadTodoistTasks,
    getTodoistConnectionStatus,
    syncPlanItem,
  } = useAppStore();

  useEffect(() => {
    loadTodoistTasks();
    getTodoistConnectionStatus();
  }, [loadTodoistTasks, getTodoistConnectionStatus]);

  const getStatusColor = () => {
    if (!todoistConnectionStatus.connected) return 'var(--color-error)';
    if (todoistConnectionStatus.rate_limited) return 'var(--color-warning)';
    return 'var(--color-success)';
  };

  const getStatusLabel = () => {
    if (!todoistConnectionStatus.connected) return 'Disconnected';
    if (todoistConnectionStatus.rate_limited) return 'Rate limited';
    return 'Connected';
  };

  const formatLastSynced = () => {
    if (!todoistConnectionStatus.last_synced_at) return 'Never';
    const date = new Date(todoistConnectionStatus.last_synced_at);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / 60000);
    if (diffMins < 1) return 'Just now';
    if (diffMins === 1) return '1 min ago';
    return `${diffMins} min ago`;
  };

  // Group tasks by due_date
  const groupedTasks: Record<string, typeof todoistTasks> = {};
  for (const task of todoistTasks) {
    const key = task.due_date || 'No date';
    if (!groupedTasks[key]) groupedTasks[key] = [];
    groupedTasks[key].push(task);
  }

  const sortedDates = Object.keys(groupedTasks).sort((a, b) => {
    if (a === 'No date') return 1;
    if (b === 'No date') return -1;
    return a.localeCompare(b);
  });

  // Plan items with todoist_task_id
  const syncedPlanItems = planItems.filter((p) => p.todoist_task_id);

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
      {/* Status row */}
      <div style={{ display: 'flex', alignItems: 'center', gap: '8px', padding: '8px 0', borderBottom: '1px solid var(--color-border)', marginBottom: '12px' }}>
        <span
          style={{
            width: '8px',
            height: '8px',
            borderRadius: '50%',
            backgroundColor: getStatusColor(),
            flexShrink: 0,
          }}
          title={getStatusLabel()}
        />
        <span style={{ fontSize: '12px', color: 'var(--color-text-muted)' }}>
          {getStatusLabel()}
        </span>
        <span style={{ fontSize: '12px', color: 'var(--color-text-muted)', marginLeft: 'auto' }}>
          Synced {formatLastSynced()}
        </span>
      </div>

      {/* Tasks grouped by date */}
      <div style={{ flex: 1, overflowY: 'auto' }}>
        {todoistTasks.length === 0 ? (
          <div style={{ textAlign: 'center', padding: '24px 0' }}>
            <p style={{ fontSize: '12px', color: 'var(--color-text-muted)' }}>No Todoist tasks</p>
            <p style={{ fontSize: '12px', color: 'var(--color-text-muted)', marginTop: '4px' }}>
              Add plan items and sync to Todoist to see them here.
            </p>
          </div>
        ) : (
          sortedDates.map((date) => (
            <div key={date} style={{ marginBottom: '16px' }}>
              <p style={{ fontSize: '12px', fontWeight: 500, color: 'var(--color-text-secondary)', marginBottom: '8px', textTransform: 'uppercase', letterSpacing: '0.05em' }}>
                {date === 'No date' ? 'No date' : date}
              </p>
              {groupedTasks[date].map((task) => {
                // Find matching plan item
                const planItem = syncedPlanItems.find((p) => p.todoist_task_id === task.id);
                return (
                  <div
                    key={task.id}
                    style={{
                      display: 'flex',
                      alignItems: 'flex-start',
                      gap: '8px',
                      padding: '8px',
                      borderRadius: 'var(--radius-sm)',
                      marginBottom: '4px',
                    }}
                  >
                    <input
                      type="checkbox"
                      checked={task.completed}
                      onChange={() => {
                        if (planItem) {
                          syncPlanItem(planItem.id);
                        }
                      }}
                      disabled={!planItem}
                      style={{ marginTop: '2px', flexShrink: 0 }}
                    />
                    <div style={{ flex: 1, minWidth: 0 }}>
                      <p
                        style={{
                          fontSize: '14px',
                          color: task.completed ? 'var(--color-text-muted)' : 'var(--color-text-primary)',
                          textDecoration: task.completed ? 'line-through' : 'none',
                          wordBreak: 'break-word',
                        }}
                      >
                        {task.content}
                      </p>
                      {task.description && (
                        <p style={{ fontSize: '12px', color: 'var(--color-text-muted)', marginTop: '2px' }}>
                          {task.description}
                        </p>
                      )}
                    </div>
                  </div>
                );
              })}
            </div>
          ))
        )}
      </div>

      {/* Manual refresh button */}
      <button
        className="btn btn-ghost"
        onClick={() => {
          loadTodoistTasks();
          getTodoistConnectionStatus();
        }}
        style={{ marginTop: '8px', fontSize: '12px' }}
      >
        Refresh
      </button>
    </div>
  );
}