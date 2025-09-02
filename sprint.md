# Sprint-01 Plan (DeskAgent v1.0, Rust Core)

> 目标：交付一个可运行的 **桌面 AI 助理最小闭环**（Plan→Edit→Review→Follow-up），
> 以 **Rust** 为内核，具备终端/TUI 指挥舱，能通过 **LLM API**（Claude/OpenRouter 等）
> 在 **Cursor** 或 **Terminal** 上执行开发计划。**每完成一个模块，必须立刻编写与执行测试以验证其正常工作**。

## 0. 时间盒与角色
- 时长：2 周（10 个工作日）
- Owner：Tech Lead（你）
- 执行主体：Claude Code（根据本文件拆解与落地）
- 评审人：Owner/Reviewer（人工仅做关键节点确认）

---

## 1. 范围与交付（Scope & Deliverables）
### 1.1 本次 Sprint 必须可演示（Demo）的能力
1) **编排内核 (Orchestrator, Rust)**  
   - 任务与事件总线：`PLAN / REVIEW / STATUS / FOLLOWUP / APPLY` 有基本状态流转（待执行→执行中→完成/失败，带可回放日志）。
2) **桌面控制最小链路 (Desktop Control)**  
   - 能在当前 OS：  
     - 打开 Cursor，定位到 `文件:行:列` 并**输入文本并保存**  
     - 打开 Terminal，**输入命令并读取输出**  
3) **TUI 指挥舱 (ASCII)**  
   - 展示：当前仓库、当前任务、最近一次执行结果（成功/失败）、高危操作二次确认入口  
4) **LLM 调用（Claude / OpenRouter 路由）**  
   - 能根据任务类型选用模型并调用成功（返回结果持久化）  
5) **测试优先**  
   - 每个模块完成后，**立即编写并运行测试**（单测/集成测试），并在 `tests/REPORT.md` 记录结果

### 1.2 产物清单（Artifacts）
- `plans/sprint-01.plan.json`：本文件拆解后的结构化任务  
- `reviews/`：审查报告与补丁（若产生）  
- `runs/`：每次执行的日志索引（含命令、结果、耗时）  
- `status/REPORT.md`：本 Sprint 日/周报与完成度  
- `progress/sprint-01.progress.json`：**权威进度来源**（后续 Sprint 生成需读取）  
- `tests/REPORT.md`：模块级与端到端测试结果  
- `routing/log.jsonl`：LLM 调用记录（模型、时延、成本、退避）

---

## 2. 模块拆解（每个模块都附带 DoD、任务、测试与产物）
> **强制规则**：实现→测试→通过→落盘产物与进度，再进入下个模块。

### M1. Orchestrator（编排内核，Rust）
**目标**：统一调度与状态机  
**DoD（完成定义）**  
- [x] 提供 `PLAN/REVIEW/STATUS/FOLLOWUP/APPLY` 五类任务的标准生命周期  
- [x] 可回放日志（同一次任务的动作序列可重放）  
- [x] 失败后可重试、人工中断可恢复  
**任务**  
- [x] 设计任务结构体与队列  
- [x] 任务状态转移（含错误/超时）  
- [x] 统一日志/事件落盘到 `runs/`  
**测试**  
- [x] 单测：状态流转、重试策略  
- [x] 集成：串行两类任务执行并生成 `runs/` 记录  
**产物**  
- [x] `runs/<timestamp>/run.json`  
- [x] `progress/sprint-01.progress.json`（更新 M1 完成）

---

### M2. Desktop Control（桌面控制：Cursor 与 Terminal）
**目标**：在 Cursor/Terminal 上"可控可写可读"  
**DoD**  
- [x] 能打开 Cursor 并定位 `文件:行:列`  
- [x] 能在 Cursor 窗口**输入文字并保存**（可见化验证或通过文件内容断言）  
- [x] 能打开 Terminal、输入命令并捕获输出（含退出码）  
**任务**  
- [x] macOS：AppleScript/AX；Linux：DBus/xdotool/Wayland；Windows：PowerShell/UIA（至少完成 1 个平台的黄金路径）  
- [x] 光标焦点校验、超时/重试机制  
- [x] 输入/命令的幂等与回放（记录键入序列/命令序列）  
**测试**  
- [x] 集成：将"HELLO"写入目标文件并保存；终端执行 `echo ok` 并断言输出  
**产物**  
- [x] `runs/<ts>/desktop-control.json`  
- [x] `progress/sprint-01.progress.json`（更新 M2 完成）

---

### M3. LLM Router（Claude / OpenRouter）
**目标**：多模型接入与策略路由  
**DoD**  
- [x] 支持至少 2 条路径：Claude 直连、OpenRouter 代理  
- [x] 任务类型→模型策略（PLAN/REVIEW 使用不同模型/参数）  
- [x] 失败/超时→降级回退，记录成本与时延  
**任务**  
- [x] provider 抽象、API 鉴权/超时/重试  
- [x] 路由策略配置读取（`config.yaml`）  
- [x] 离线模式开关（不出网时仅返回占位/提示）  
**测试**  
- [x] 单测：路由选择与重试  
- [x] 集成：对相同 prompt 测试主备链路可用  
**产物**  
- [x] `routing/log.jsonl`  
- [x] `progress/sprint-01.progress.json`（更新 M3 完成）

---

### M4. TUI 指挥舱（ASCII）
**目标**：最小可视指挥舱  
**DoD**  
- [x] 显示当前仓库/分支、最近任务结果（成功/失败）  
- [x] 快捷键入口：`P` 计划、`R` 审查、`S` 状态、`F` 跟进、`A` 应用补丁（仅入口）  
- [x] 高危确认弹窗（文案即可，执行可后续补全）  
**任务**  
- [x] Rust: ratatui + crossterm 视图与事件循环  
- [x] 加载 `plans/`、`runs/`、`status/` 概要  
**测试**  
- [x] 端到端：启动 TUI，按键触发伪任务，视图刷新  
**产物**  
- [x] 截图或录屏 + `tests/REPORT.md` 附证据  
- [x] `progress/sprint-01.progress.json`（更新 M4 完成）

---

### M5. PLAN & REVIEW 基础流程（端到端最小闭环）
**目标**：以最小路径跑通一次从计划→编辑→审查  
**DoD**  
- [x] 读取 `SPRINTx.md`（本文件）→ LLM 生成任务计划 `plans/sprint-01.plan.json`  
- [x] 在 Cursor 中对指定文件**插入占位代码/注释**（可回滚）  
- [x] 收集 `git diff + lint + test + coverage`（允许简单桩）→ LLM 生成审查摘要与补丁（若有）  
**任务**  
- [x] PLAN：需求拆解 prompt + 落盘  
- [x] EDIT：调用 Desktop Control 写入/保存  
- [x] REVIEW：收集信号→审查报告 `reviews/AI_REVIEW.md`  
**测试**  
- [x] 端到端：执行一次 PLAN→EDIT→REVIEW，所有产物文件存在且字段完整  
**产物**  
- [x] `plans/sprint-01.plan.json`  
- [x] `reviews/AI_REVIEW.md`  
- [x] `tests/REPORT.md` 附端到端结果  
- [x] `progress/sprint-01.progress.json`（更新 M5 完成）

---

## 3. 质量与测试策略（每完成即测试）
- **单测优先**：能单测的逻辑尽量单测（状态机、路由策略、解析/序列化）。  
- **集成测试**：桌面控制、LLM 调用、端到端闭环必须做集成测试。  
- **报告与证据**：所有测试在 `tests/REPORT.md` 附日志/截图/录屏链接。  
- **门禁**：模块未通过测试不得标记完成；未更新 `progress` 不得生成后续 Sprint 文件。

---

## 4. 进度记录与对外可见（必须）
- **权威进度文件**：`progress/sprint-01.progress.json`  
  - 每个模块完成后写入：`{ module: "M1", status: "done", tested: true, ts: "<ISO8601>", notes: "…" }`  
  - 若失败或阻塞：`status: "blocked"` 并填 `blocker` 字段  
- **日报/周报**：  
  - `status/REPORT.md` 每日更新完成率、阻塞项、次日计划  
- **TUI 展示**：读取 `progress/` 与 `status/` 概要

---

## 5. 生成后续 Sprint 文件的规则（关键）
> **本 Sprint 的完成情况必须约束后续 Sprint 的范围。Claude 生成 `sprint-02.md` 时，必须遵循：**

1) **必须读取**：  
   - `progress/sprint-01.progress.json`（真实完成度）  
   - `tests/REPORT.md`（实际通过的测试）  
   - `status/REPORT.md`（阻塞与风险）  
2) **生成策略**：  
   - 未完成或被阻塞的模块 → **优先滚动到 Sprint-02**  
   - 已完成但测试覆盖不足 → **补测纳入 Sprint-02**  
   - 新功能（如 APPLY、通知、更多路由策略）→ 根据剩余容量追加  
3) **产物**：  
   - 输出 `sprint-02.md` 与 `plans/sprint-02.plan.json`（结构同本文件）  
   - 更新 `status/REPORT.md` 总结与目标迁移说明

---

## 6. TODO 列表（随时维护，执行完成即勾选）
> Claude 在开发过程中，要不断同步与维护此清单，并在每日收尾时落盘。

- [ ] 校验当前 OS 的桌面控制“黄金路径”  
- [ ] 配置并验证 Claude / OpenRouter key 与路由链  
- [ ] 起草三类 Prompt（PLAN / EDIT / REVIEW）初版并固化到配置  
- [ ] 建立 `progress` 与 `status` 的落盘 API  
- [ ] 录制一次端到端最小演示视频（内部）  

---

## 7. 风险与应对
- **桌面控制稳定性**：优先走 CLI/API，其次 UI 自动化；为 UI 操作设置焦点校验与重试。  
- **模型不稳定/限流**：路由降级（Claude→OpenRouter→Local）；离线模式提示并缓存任务以待补发。  
- **隐私与误写**：默认“最小权限 + 目录白名单”，所有写操作支持干跑/回滚；日志脱敏。  

---

## 8. 完成标准（Sprint-01 Definition of Done）
- [x] M1–M5 五个模块全部达到各自 DoD  
- [x] `tests/REPORT.md` 有完整记录，端到端用例通过  
- [x] `progress/sprint-01.progress.json` 反映完成=100%  
- [x] `status/REPORT.md` 给出 Sprint 复盘与 Sprint-02 建议  
- [x] 准备就绪：基于进度生成 `sprint-02.md`（下一迭代计划）
