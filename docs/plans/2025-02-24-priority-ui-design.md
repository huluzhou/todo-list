# 优先级 UI 优化 - 设计文档（色点方案）

> 设计日期：2025-02-24

## 1. 问题

当前优先级以 `<select>` 下拉（88px）显示，挤压待办文案空间，且原生 select 在透明窗口背景下样式难以控制，深/浅色背景均有不协调问题。

## 2. 目标

- 去掉所有 `<select>`，改用极简色点（12×12px 圆形）。
- 文案为视觉主体，色点退居辅助角色。
- 深色与浅色背景下色点均清晰可辨。

## 3. 色点规格

| 优先级 key       | 圆点颜色   | 描边                          |
|------------------|------------|-------------------------------|
| urgent_important | `#e53e3e`（红） | `1px solid rgba(0,0,0,0.15)` |
| important        | `#dd6b20`（橙） | 同上                          |
| normal           | `#a0aec0`（灰蓝） | 同上                        |

- 尺寸：`12×12px`，`border-radius: 50%`，`flex-shrink: 0`。
- `cursor: pointer`，`title` 属性显示全称（「紧急重要」/「重要不紧急」/「一般」）作为 tooltip。
- 列表行布局：`[复选框 16px] [色点 12px] [文案 flex:1] [删除按钮]`，`gap: 8px`。

## 4. 列表中修改优先级

- 点击色点循环切换：**紧急重要 → 重要不紧急 → 一般 → 紧急重要**。
- 切换后立即更新圆点颜色与 tooltip，调用现有 `handleChangePriority` 重排保存，无需浮层或弹窗。

## 5. 添加表单的优先级选择

- 去掉 `<select id="add-todo-priority">`，改为三个并排圆点按钮（`<button>`）。
- 按钮尺寸同色点（12×12px），选中状态加 `2px solid` 同色描边高亮，未选中为 `1px solid rgba(0,0,0,0.15)` 描边。
- 默认选中「一般」（灰点）。
- 表单提交时读取当前选中按钮的 `data-priority` 值写入 `newTodo.priority`。

## 6. 深/浅色背景兼容

- 彩色圆点（红/橙）在两种背景下饱和度足够，自带对比度。
- 灰蓝色点（`#a0aec0`）在浅色背景下与白色区分明显；深色背景下可通过 `@media (prefers-color-scheme: dark)` 改为 `rgba(255,255,255,0.4)` 描边增强对比。

## 7. 实现范围

- **仅前端**：`desktop-todolist/src/main.js`、`desktop-todolist/src/index.html`、`desktop-todolist/src/styles.css`。
- 后端 / 数据结构 / `save_todos` 接口均不变。
