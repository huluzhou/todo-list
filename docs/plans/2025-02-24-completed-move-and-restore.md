# 完成移底与取消完成恢复位置 - 实施计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 勾选完成时该项自动移到底部，取消完成时该项恢复到列表最上方（第一条）。

**Architecture:** 仅前端：展示按 (done, order) 排序（未完成在前）；切换完成状态时更新 done 后重排 order 为 0..n-1 再持久化。

**Tech Stack:** 现有 desktop-todolist 前端（Vanilla JS）、save_todos invoke。

**设计文档:** `docs/plans/2025-02-24-completed-move-and-restore-design.md`

---

### Task 1: 展示顺序改为「未完成在前、已完成在后」

**文件:**
- 修改: `desktop-todolist/src/main.js`（`sortedTodos` 函数）

**Step 1: 修改 sortedTodos 的排序规则**

当前为仅按 `order` 升序。改为先按 `done`（false 在前、true 在后），再按 `order` 升序：

```javascript
function sortedTodos(list) {
  return [...(list || [])].sort((a, b) => {
    if (a.done !== b.done) return (a.done ? 1 : 0) - (b.done ? 1 : 0);
    return (a.order ?? 0) - (b.order ?? 0);
  });
}
```

**Step 2: 本地验证**

在项目根执行 `cd desktop-todolist && npm run tauri dev`（或在浏览器中打开前端），确认列表展示为：未完成在上、已完成在下；同状态内按 order 顺序不变。

**Step 3: 提交**

```bash
git add desktop-todolist/src/main.js
git commit -m "feat(ui): 列表按未完成在前、已完成在后排序"
```

---

### Task 2: 切换完成状态时重排 order 并持久化

**文件:**
- 修改: `desktop-todolist/src/main.js`（`handleToggleDone` 函数）

**Step 1: 在 handleToggleDone 中增加重排 order 逻辑**

在 `item.done = !!done` 之后、`saveTodosAndRender` 之前：

1. 用当前 `sortedTodos(todos)` 得到目标显示顺序（未完成在前、已完成在后）。
2. 按该顺序依次给每项赋 `order = 0, 1, …, n-1`（直接改 `todos` 中对象的 `order` 属性）。
3. 再调用 `saveTodosAndRender(listEl)`（内部会 save_todos 并重渲染）。

示例实现：

```javascript
async function handleToggleDone(id, done, listEl) {
  const item = todos.find((t) => t.id === id);
  if (!item) return;
  item.done = !!done;
  const sorted = sortedTodos(todos);
  sorted.forEach((t, i) => {
    t.order = i;
  });
  await saveTodosAndRender(listEl);
}
```

**Step 2: 本地验证**

- 勾选一项为完成 → 该项应移到底部。
- 再取消勾选 → 该项应回到列表最上方（第一条）。

**Step 3: 提交**

```bash
git add desktop-todolist/src/main.js
git commit -m "feat(ui): 完成移到底部、取消完成恢复到最上方"
```

---

## 执行选项

计划已保存至 `docs/plans/2025-02-24-completed-move-and-restore.md`。

**两种执行方式：**

1. **本会话子 agent 驱动** — 在本会话中按任务派发子 agent，每步完成后审查，迭代快。
2. **独立会话** — 在新会话中打开项目，使用 @superpowers:executing-plans 按任务执行并设置检查点。

你更倾向哪一种？
