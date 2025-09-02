# Review Workspace Mockup

## ASCII Layout

```
┌─ Code Review Workspace ──────────────────────────────────────┐
│ Review ID: AI_REVIEW_20250902_001 │ Status: ✅ APPROVED      │
│ Quality Score: 95/100 │ Reviewer: Claude AI │ Time: 2.3s    │
├──────────────────────────────────────────────────────────────┤
│ File Changes (12 files modified):                           │
│ ┌─────────────────────┬──────┬──────┬───────────────────────┐ │
│ │ File                │ +/-  │ Risk │ Issues               │ │
│ ├─────────────────────┼──────┼──────┼───────────────────────┤ │
│ │ src/orchestrator/   │ +45  │ 🟡   │ Complex state logic  │ │
│ │ src/desktop/cursor  │ +32  │ 🔴   │ AppleScript safety   │ │
│ │ src/llm/router.rs   │ +28  │ 🟢   │ Clean implementation │ │
│ │ src/tui/mod.rs      │ +156 │ 🟡   │ Large UI module      │ │
│ │ src/workflows/edit  │ +89  │ 🟢   │ Good error handling  │ │
│ │ ...                 │ ...  │ ...  │ ...                  │ │
│ └─────────────────────┴──────┴──────┴───────────────────────┘ │
├──────────────────────────────────────────────────────────────┤
│ Analysis Summary:                                            │
│ ✅ Architecture: Clean modular design with proper separation │
│ ✅ Error Handling: Comprehensive Result<T> patterns         │
│ ✅ Testing: 98.4% test coverage, all critical paths tested  │
│ ⚠️  Performance: Some blocking I/O in async contexts        │
│ ⚠️  Security: API keys hardcoded, need secure storage       │
├──────────────────────────────────────────────────────────────┤
│ Recommendations:                                             │
│ 1. Move API key management to secure storage                │
│ 2. Replace blocking I/O with async alternatives             │
│ 3. Add input validation for AppleScript commands            │
│ 4. Consider rate limiting for LLM API calls                 │
├──────────────────────────────────────────────────────────────┤
│ Actions: [Approve] [Request Changes] [Export Report] [Diff] │
└──────────────────────────────────────────────────────────────┘
```

## Interactive Elements

### Header
- **Review ID**: Unique identifier with timestamp
- **Status Badge**: APPROVED, PENDING, CHANGES_REQUESTED
- **Metrics**: Quality score, reviewer, review time

### File Changes Table
- **Sortable Columns**: Click headers to sort by lines, risk, etc.
- **Risk Color Coding**: 🔴 High, 🟡 Medium, 🟢 Low
- **Expandable Rows**: Click file to see detailed changes
- **Issue Preview**: Hover for detailed issue description

### Analysis Summary
- **Category Breakdown**: Architecture, testing, performance, security
- **Status Indicators**: ✅ (good), ⚠️ (warning), ❌ (critical)
- **Expandable Sections**: Click to see detailed analysis

### Recommendations
- **Numbered List**: Priority-ordered suggestions
- **Action Items**: Each recommendation is actionable
- **Severity Indicators**: Critical vs nice-to-have improvements

### Action Bar
- **Review Actions**: Approve or request changes
- **Export**: Generate review report
- **Diff Viewer**: Open file diff in separate view

## Data Bindings

- **Review Data**: From `reviews/AI_REVIEW.md`
- **File Changes**: From git diff analysis
- **Quality Metrics**: From automated code analysis
- **Risk Assessment**: From security and complexity analysis
- **Recommendations**: From AI-generated suggestions
- **Test Coverage**: From test execution reports