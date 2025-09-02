# Dashboard Mockup

## ASCII Layout

```
┌─ DeskAgent GUI ──────────────────────────────────────────────┐
│ 🏠 Repo: deskagent v1.0   📍 Branch: main                   │
│ 📊 Tasks: 12 total │ 8 done │ 4 pending │ 0 failed        │
├──────────────────────────────────────────────────────────────┤
│ Sprint Progress: [██████████░░░░░░░░░░] 52%                 │
│ Last Review: ✅ PASS │ Risks: 🔴 2 HIGH, 🟡 3 MEDIUM        │
├──────────────────────────────────────────────────────────────┤
│ Recent Activity:                                             │
│ • 15:42 M5 Workflow Integration ✅                           │
│ • 14:23 M4 TUI Dashboard ✅                                  │
│ • 13:15 M3 LLM Router ✅                                     │
│ • 12:01 M2 Desktop Control ✅                                │
├──────────────────────────────────────────────────────────────┤
│ Quick Actions:                                               │
│ [P] Plan  [R] Review  [S] Status  [F] Follow  [A] Apply    │
│ [N] Notify [O] Offline [Q] Quit   [H] Help                  │
└──────────────────────────────────────────────────────────────┘
```

## Interactive Elements

### Top Bar
- **Repository Info**: Shows current repo name and version
- **Branch**: Current git branch with status indicator
- **Task Summary**: Real-time count of task states

### Progress Section
- **Visual Progress Bar**: ASCII progress bar with percentage
- **Review Status**: Last review result with timestamp
- **Risk Indicators**: Color-coded risk levels from reviews

### Activity Feed
- **Recent Tasks**: Shows last 4 completed/active tasks
- **Status Icons**: ✅ (success), ❌ (failed), 🔄 (running), ⏸️ (paused)
- **Timestamps**: When each task was completed

### Action Bar
- **Keyboard Shortcuts**: Single-key navigation
- **Context Sensitive**: Some actions only available when relevant

## Data Bindings

- **Repository**: From git status
- **Tasks**: From `progress/sprint-01.progress.json`
- **Progress**: Calculated from completed modules
- **Reviews**: From `reviews/AI_REVIEW.md`
- **Activity**: From `runs/` directory logs