
1. 先完成 **核心内核开发与测试**（Orchestrator、Desktop Control、LLM Router、TUI、PLAN/REVIEW 闭环）。
2. 在核心功能验证可运行之后，继续进行 **GUI 设计与实现**（Mockup → Implementation），同时补齐跨平台桌面控制与测试覆盖率。


---

```markdown
# Sprint-01 Plan (DeskAgent v1.0, Rust Core + GUI)

> 目标：交付一个可运行的 **桌面 AI 助理闭环**（Plan→Edit→Review→Follow-up），
> 以 **Rust** 为内核，先完成模块开发与测试，再开展 GUI 设计与实现。
> 每完成一个模块必须立即测试并更新进度文件，避免未测代码进入 GUI 阶段。

---

## 0. 时间盒与角色
- 时长：4 周（分两个阶段，每阶段 2 周）
- Owner：Tech Lead（你）
- 执行主体：Claude Code
- 评审人：Owner/Reviewer

---

## 1. 范围与交付
1. **阶段一：核心内核开发与测试**
   - Orchestrator（编排内核）
   - Desktop Control（Cursor + Terminal）
   - LLM Router（Claude / OpenRouter）
   - TUI 指挥舱（ASCII Dashboard）
   - PLAN & REVIEW 最小闭环  
   **产物**：能跑通一次端到端 Demo，所有模块均有测试

2. **阶段二：GUI 设计与实现**
   - GUI Mockup (Dashboard / Sprint 面板 / 审查工作台)
   - GUI Implementation（Rust + Tauri/ratatui）
   - 跨平台桌面控制验证
   - PLAN & REVIEW 覆盖率 ≥ 80%  
   **产物**：GUI 可展示真实数据，覆盖率报告可用

---

## 2. 模块拆解（阶段一：核心内核）

### M1. Orchestrator（编排内核）
- 任务总线：`PLAN/REVIEW/STATUS/FOLLOWUP/APPLY`
- 状态机：待执行→执行中→完成/失败
- 日志落盘到 `runs/`
- ✅ 测试：状态流转、重试、日志生成

---

### M2. Desktop Control（Cursor + Terminal）
- 打开 Cursor，定位 `文件:行:列` 并输入文字保存
- 打开 Terminal，输入命令并捕获输出
- ✅ 测试：写入“HELLO”到文件并保存；执行 `echo ok` 并断言输出

---

### M3. LLM Router
- Claude 直连 + OpenRouter 路由
- 任务类型→模型策略
- 限流/超时→降级
- ✅ 测试：Claude 与 OpenRouter 均可调用成功

---

### M4. TUI 指挥舱
- 显示当前仓库、任务、执行结果
- 快捷键入口：`P`/`R`/`S`/`F`/`A`
- ✅ 测试：模拟任务执行，界面刷新

---

### M5. PLAN & REVIEW 闭环
- PLAN：解析 `SPRINTx.md` → `plans/sprint-01.plan.json`
- EDIT：调用 Desktop Control 修改文件
- REVIEW：收集 `git diff + lint + test` → Claude 输出 `reviews/AI_REVIEW.md`
- ✅ 测试：端到端一次执行，产物完整

---

## 3. 模块拆解（阶段二：GUI 设计与实现）

### M6. GUI Mockup & UX Design
- 输出 3 个视图：Dashboard / Sprint 面板 / 审查工作台
- Mockup 存放于 `gui/mockups/`
- 交互逻辑写入 `gui/DESIGN.md`
- ✅ 测试：人工验收 Mockup 完整度

**ASCII Mockup 示例 (Dashboard)**  
```

┌─ DeskAgent GUI ──────────────────────────────┐
│ Repo: mvp-app   Branch: feat/sprint1         │
│ Tasks: 12 total | 8 done | 4 pending         │
├──────────────────────────────────────────────┤
│ Sprint Progress: \[██████░░░░] 67%            │
│ Last Review: PASS | Risks: 2 HIGH, 3 MEDIUM │
├──────────────────────────────────────────────┤
│ \[P] Plan  \[R] Review  \[S] Status  \[F] Follow │
│ \[A] Apply \[N] Notify \[O] Offline  \[Q] Quit  │
└──────────────────────────────────────────────┘

```

---

### M7. GUI Implementation (Rust + Tauri/ratatui)
- Dashboard：展示仓库、进度、执行结果
- Sprint 面板：任务树可展开/收起
- 审查工作台：文件列表、diff 摘要、风险等级
- 界面与 `progress/`、`status/` 文件绑定
- ✅ 测试：运行 Demo，GUI 展示真实数据

---

### M8. Cross-Platform Desktop Control
- 验证桌面控制在 macOS/Linux/Windows 可用
- 输出 `docs/desktop-control.md`
- ✅ 测试：三平台均执行“写入文件+echo ok”

---

### M9. PLAN & REVIEW Coverage
- 提升单测覆盖率 ≥ 80%
- 增补测试样例
- ✅ 测试：输出 `tests/coverage/plan-review.html`

---

## 4. 产物清单
- `plans/sprint-01.plan.json`
- `reviews/AI_REVIEW.md`
- `runs/<ts>/run.json`
- `status/REPORT.md`
- `progress/sprint-01.progress.json`
- `tests/REPORT.md`
- `gui/mockups/*.png|.md`
- `gui/DESIGN.md`
- `src/gui/` GUI 源码
- `tests/coverage/plan-review.html`

---

## 5. TODO 列表
- [ ] 实现 Orchestrator 内核并测试  
- [ ] 完成 Desktop Control 并验证跨平台  
- [ ] 实现 LLM Router，确认 Claude 与 OpenRouter 可用  
- [ ] 完成 TUI 指挥舱并跑通 PLAN/REVIEW 闭环  
- [ ] 输出 GUI Mockup 并写设计文档  
- [ ] 实现 GUI 模块并绑定进度文件  
- [ ] 提升 PLAN/REVIEW 覆盖率 ≥ 80%  
- [ ] 录制一次端到端 Demo 并附入 `tests/REPORT.md`

---

## 6. 完成标准
- 阶段一 (内核) 与阶段二 (GUI) 的所有模块全部通过测试  
- `progress/sprint-01.progress.json` 与 `status/REPORT.md` 更新至 100%  
- PLAN/REVIEW 覆盖率 ≥ 80%  
- GUI Mockup 与实现完成，可展示真实数据  
- 具备端到端 Demo，准备进入 Sprint-02 (扩展 APPLY、通知系统等)
```

