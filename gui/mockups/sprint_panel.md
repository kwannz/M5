# Sprint Panel Mockup

## ASCII Layout

```
┌─ Sprint Management Panel ────────────────────────────────────┐
│ Sprint-01: DeskAgent v1.0 Core Implementation               │
│ Status: ✅ COMPLETED │ Duration: 1 day │ Progress: 100%     │
├──────────────────────────────────────────────────────────────┤
│ Module Tree:                                                 │
│ ├─ 📦 M1: Orchestrator                          ✅ [11/11]  │
│ │  ├─ Task State Machine                        ✅           │
│ │  ├─ Event Logging                             ✅           │
│ │  └─ Retry Logic                               ✅           │
│ ├─ 📦 M2: Desktop Control                       ✅ [17/17]  │
│ │  ├─ AppleScript Integration                   ✅           │
│ │  ├─ Cursor IDE Control                        ✅           │
│ │  └─ Terminal Automation                       ✅           │
│ ├─ 📦 M3: LLM Router                            ✅ [13/13]  │
│ │  ├─ Claude Provider                           ✅           │
│ │  ├─ OpenRouter Provider                       ✅           │
│ │  └─ Routing Strategy                          ✅           │
│ ├─ 📦 M4: TUI Dashboard                         ✅ [3/3]    │
│ │  ├─ ratatui Interface                         ✅           │
│ │  ├─ Keyboard Shortcuts                        ✅           │
│ │  └─ Status Display                            ✅           │
│ └─ 📦 M5: Workflow Integration                  ✅ [12/12]  │
│    ├─ PLAN Workflow                             ✅           │
│    ├─ EDIT Workflow                             ✅           │
│    └─ REVIEW Workflow                           ✅           │
├──────────────────────────────────────────────────────────────┤
│ Deliverables:                                                │
│ • plans/sprint-01.plan.json                    ✅           │
│ • reviews/AI_REVIEW.md                         ✅           │
│ • progress/sprint-01.progress.json             ✅           │
│ • status/REPORT.md                             ✅           │
│ • routing/log.jsonl                            ✅           │
├──────────────────────────────────────────────────────────────┤
│ Actions: [Expand All] [Collapse All] [Export] [Next Sprint] │
└──────────────────────────────────────────────────────────────┘
```

## Interactive Elements

### Header
- **Sprint Title**: Current sprint identifier
- **Status Badge**: Overall completion status
- **Metrics**: Duration and progress percentage

### Module Tree
- **Expandable Nodes**: Click to expand/collapse module details
- **Status Icons**: ✅ (done), 🔄 (in progress), ❌ (failed), ⏸️ (paused)
- **Test Counters**: [passed/total] for each module
- **Sub-tasks**: Indented list of module components

### Deliverables Section
- **File Status**: Real-time check of required deliverable files
- **Link Actions**: Click to open files in editor
- **Validation**: Green checkmarks for complete deliverables

### Action Bar
- **Tree Controls**: Expand/collapse all nodes
- **Export**: Generate sprint report
- **Navigation**: Move to next sprint planning

## Data Bindings

- **Sprint Info**: From `progress/sprint-01.progress.json`
- **Module Status**: From progress tracking
- **Test Results**: From test execution results
- **Deliverables**: File system checks for required artifacts
- **Timestamps**: From module completion dates