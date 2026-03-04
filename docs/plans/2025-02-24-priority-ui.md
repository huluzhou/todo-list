# 优先级 UI 优化（色点方案）Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 将列表和添加表单中的 `<select>` 优先级下拉替换为 12×12px 纯色圆点，列表中点击圆点循环切换优先级，添加表单中三个圆点并排作为单选按钮，彻底消灭 `<select>` 样式问题，文案成为视觉主体。

**Architecture:** 仅前端改动：`main.js` 的 `renderTodos`（用 `<button class="todo-priority-dot">` 替代 `<select>`）、`handleAddTodo`（读取表单中选中的圆点按钮）、事件委托（click 代替 change）；`index.html` 表单中替换 select；`styles.css` 新增色点与选中态样式，移除旧 select 样式。后端/数据结构不变。

**Tech Stack:** Vanilla JS, CSS, Tauri 2（HTML）。设计见 `docs/plans/2025-02-24-priority-ui-design.md`

---

### Task 1: 列表中的 `<select>` 替换为色点按钮

**Files:**
- Modify: `desktop-todolist/src/main.js`（`renderTodos` 函数）
- Modify: `desktop-todolist/src/styles.css`（新增色点样式，移除旧 select 样式）

**Step 1: 在 main.js 中定义优先级色点配置**

在 `PRIORITY_ORDER` 常量附近增加：

```javascript
/** 优先级对应的色点颜色与 tooltip */
const PRIORITY_DOT = {
  urgent_important: { color: "#e53e3e", label: "紧急重要" },
  important:        { color: "#dd6b20", label: "重要不紧急" },
  normal:           { color: "#a0aec0", label: "一般" },
};
```

**Step 2: 修改 `renderTodos` 中的优先级元素**

将 `prioritySelect`（`<select>`）创建逻辑替换为：

```javascript
const p = todo.priority ?? "normal";
const dotCfg = PRIORITY_DOT[p] ?? PRIORITY_DOT.normal;
const priorityBtn = document.createElement("button");
priorityBtn.type = "button";
priorityBtn.className = "todo-priority-dot";
priorityBtn.dataset.id = todo.id;
priorityBtn.title = dotCfg.label;
priorityBtn.style.backgroundColor = dotCfg.color;
```

并将 `li.appendChild(prioritySelect)` 改为 `li.appendChild(priorityBtn)`。

**Step 3: 修改事件委托（click 替代 change）**

在 `listEl.addEventListener("change", ...)` 中，移除 `.todo-priority` 的 change 处理。

在 `listEl.addEventListener("click", ...)` 中，增加对 `.todo-priority-dot` 的处理：

```javascript
const dotBtn = e.target.closest(".todo-priority-dot");
if (dotBtn) {
  const id = dotBtn.dataset.id;
  if (!id) return;
  const item = todos.find((t) => t.id === id);
  if (!item) return;
  // 循环切换：urgent_important -> important -> normal -> urgent_important
  const cycle = ["urgent_important", "important", "normal"];
  const cur = cycle.indexOf(item.priority ?? "normal");
  const next = cycle[(cur + 1) % cycle.length];
  handleChangePriority(id, next, listEl);
  return;
}
```

（放在 `deleteBtn` 的 `return` 之后、`textSpan` 判断之前）

**Step 4: 为色点新增 CSS，移除旧 select 样式**

在 `styles.css` 中：
1. 删除「列表项内优先级下拉」块（`.todo-priority` 的 88px select 样式）。
2. 新增：

```css
/* 列表项内优先级色点 */
#todo-list > li .todo-priority-dot,
.todo-list > li .todo-priority-dot {
  flex-shrink: 0;
  width: 12px;
  height: 12px;
  border-radius: 50%;
  border: 1px solid rgba(0, 0, 0, 0.15);
  padding: 0;
  cursor: pointer;
  background-color: #a0aec0; /* 默认，由 JS 覆盖 */
  transition: transform 0.1s, box-shadow 0.1s;
  margin-top: 4px; /* 与文案垂直对齐 */
}

#todo-list > li .todo-priority-dot:hover,
.todo-list > li .todo-priority-dot:hover {
  transform: scale(1.3);
  box-shadow: 0 0 0 2px rgba(0, 0, 0, 0.1);
}

@media (prefers-color-scheme: dark) {
  #todo-list > li .todo-priority-dot,
  .todo-list > li .todo-priority-dot {
    border-color: rgba(255, 255, 255, 0.25);
  }
}
```

**Step 5: 手动验证**

运行 `cd desktop-todolist && npm run tauri dev`，确认：
- 列表每项有色点（红/橙/灰），文案区域明显变宽。
- 点击色点循环切换颜色并重排。
- 深色背景下色点清晰可辨。

**Step 6: Commit**

```bash
git add desktop-todolist/src/main.js desktop-todolist/src/styles.css
git commit -m "feat(ui): 列表优先级改为色点，点击循环切换"
```

---

### Task 2: 添加表单的 `<select>` 替换为三色点单选

**Files:**
- Modify: `desktop-todolist/src/index.html`（移除 select，添加三色点按钮组）
- Modify: `desktop-todolist/src/main.js`（`handleAddTodo` 读取选中色点，添加色点选中逻辑）
- Modify: `desktop-todolist/src/styles.css`（新增表单色点样式）

**Step 1: 修改 `index.html`**

将表单中的 `<select id="add-todo-priority">...</select>` 替换为：

```html
<div id="add-todo-priority-group" class="priority-dot-group" role="group" aria-label="优先级">
  <button type="button" class="priority-dot-btn" data-priority="urgent_important"
    title="紧急重要" style="background-color:#e53e3e;"></button>
  <button type="button" class="priority-dot-btn" data-priority="important"
    title="重要不紧急" style="background-color:#dd6b20;"></button>
  <button type="button" class="priority-dot-btn is-selected" data-priority="normal"
    title="一般" style="background-color:#a0aec0;"></button>
</div>
```

**Step 2: 修改 `handleAddTodo`**

将读取 `priorityEl?.value` 改为：

```javascript
const selectedDot = form.querySelector(".priority-dot-btn.is-selected");
const priority =
  selectedDot?.dataset?.priority && PRIORITY_ORDER[selectedDot.dataset.priority] !== undefined
    ? selectedDot.dataset.priority
    : "normal";
```

**Step 3: 添加表单色点的点击切换逻辑**

在 `DOMContentLoaded` 中，在 `form.addEventListener("submit", ...)` 之前增加：

```javascript
// 优先级色点单选：点击切换 is-selected
if (form) {
  form.addEventListener("click", (e) => {
    const dotBtn = e.target.closest(".priority-dot-btn");
    if (!dotBtn) return;
    form.querySelectorAll(".priority-dot-btn").forEach((b) => b.classList.remove("is-selected"));
    dotBtn.classList.add("is-selected");
  });
}
```

**Step 4: 新增表单色点 CSS**

在 `styles.css` 中，删除 `#add-todo-priority` 的 select 样式，新增：

```css
/* 添加表单优先级色点组 */
.priority-dot-group {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  gap: 5px;
}

.priority-dot-btn {
  width: 14px;
  height: 14px;
  border-radius: 50%;
  border: 1px solid rgba(0, 0, 0, 0.15);
  padding: 0;
  cursor: pointer;
  transition: transform 0.1s, box-shadow 0.1s;
}

.priority-dot-btn:hover {
  transform: scale(1.2);
}

/* 选中态：同色 2px 实线描边 */
.priority-dot-btn.is-selected {
  box-shadow: 0 0 0 2px #fff, 0 0 0 4px currentColor;
  outline: none;
}

@media (prefers-color-scheme: dark) {
  .priority-dot-btn {
    border-color: rgba(255, 255, 255, 0.25);
  }
  .priority-dot-btn.is-selected {
    box-shadow: 0 0 0 2px #1e1e1e, 0 0 0 4px currentColor;
  }
}
```

> 注意：`currentColor` 在 `<button>` 上取 `color` 值，但色点用 `background-color` 着色，`currentColor` 会是黑/白。需改为用 JS 读取 `style.backgroundColor` 并 set 到 `outline` 或 `box-shadow`，或改用固定描边色（与色点颜色对应）。**简化方案**：选中态改用 `outline: 2px solid rgba(0,0,0,0.4)` + `outline-offset: 2px`，在深浅背景均有效，无需 JS 处理颜色。

最终 `is-selected` CSS 改为：

```css
.priority-dot-btn.is-selected {
  outline: 2px solid rgba(0, 0, 0, 0.35);
  outline-offset: 2px;
}

@media (prefers-color-scheme: dark) {
  .priority-dot-btn.is-selected {
    outline-color: rgba(255, 255, 255, 0.5);
  }
}
```

**Step 5: 手动验证**

运行应用，确认：
- 添加表单右侧显示三个色点（红/橙/灰），默认灰点选中（有外描边）。
- 点击某色点后该点加描边、其余去描边。
- 提交后新待办的优先级正确，排序符合预期。

**Step 6: Commit**

```bash
git add desktop-todolist/src/index.html desktop-todolist/src/main.js desktop-todolist/src/styles.css
git commit -m "feat(ui): 添加表单优先级改为三色点单选"
```

---

## 执行方式

计划已保存到 `docs/plans/2025-02-24-priority-ui.md`。两种执行方式：

**1. Subagent-Driven（本会话）** — 按任务派发子 agent，每步 review，快速迭代。
   **所需子技能：** 使用 superpowers:subagent-driven-development。

**2. Parallel Session（另开会话）** — 独立会话中用 executing-plans 批次执行。
   **所需子技能：** 新会话使用 superpowers:executing-plans。

你选哪一种？
