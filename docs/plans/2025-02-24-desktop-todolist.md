# Windows 桌面极简 Todolist 实施计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 在 Windows 上实现一个极简桌面 Todolist：半透明无边框小窗、置顶可切换、记位可拖、固定大小，增删查改与完成状态，数据存本地 JSON，支持开机启动。

**Architecture:** Tauri 2 应用，Rust 负责窗口行为与 JSON 读写，前端负责 UI 与交互；待办存 `todos.json`，窗口位置与置顶偏好存 `window.json`，均在 app_data_dir 下。

**Tech Stack:** Tauri 2.x、Rust、Vanilla HTML/CSS/JS；Windows 启动项（启动文件夹或 Run 注册表）。

**设计文档:** `docs/plans/2025-02-24-desktop-todolist-design.md`

---

### Task 1: 搭建 Tauri 2 项目骨架

**文件:**
- 创建: 由 `create-tauri-app` 生成整项目（如 `todo-list` 根目录下已有仓库，可在其内执行或先建子目录再迁入）
- 参考: [Tauri 2 Create Project](https://v2.tauri.app/start/create-project/)

**Step 1: 使用 create-tauri-app 创建项目**

在仓库根目录或目标目录执行（选 Vanilla + JavaScript，项目名如 `desktop-todolist` 或与现有 repo 一致）：

```bash
npm create tauri-app@latest
```

交互选择：Project name → Identifier → 前端语言选 **TypeScript/JavaScript** → 包管理选 **npm** → UI 模板选 **Vanilla** → 语言选 **JavaScript**。

**Step 2: 安装依赖并确认能启动**

```bash
cd <项目名>
npm install
npm run tauri dev
```

预期：弹出默认 Tauri 窗口，内容为模板页。

**Step 3: 提交**

```bash
git add .
git commit -m "chore: 搭建 Tauri 2 Vanilla 项目骨架"
```

---

### Task 2: 配置窗口（无边框、半透明、固定大小）

**文件:**
- 修改: `src-tauri/tauri.conf.json`（或 `src-tauri/tauri.conf.json5`，视模板而定）

**Step 1: 在 app > windows 中设置主窗口**

将默认主窗口配置改为：

- `decorations: false`
- `transparent: true`
- `resizable: false`
- `width: 320`, `height: 400`
- 标题可设为 `待办` 或 `Todolist`

（具体键名以 Tauri 2 官方 [Configuration](https://v2.tauri.app/reference/config/) 为准；若为数组则改第一个窗口对象。）

**Step 2: 再次运行确认**

```bash
npm run tauri dev
```

预期：窗口无边框、半透明、固定 320×400，不可拉大缩小。

**Step 3: 提交**

```bash
git add src-tauri/tauri.conf.json
git commit -m "feat(窗口): 无边框、半透明、固定大小"
```

---

### Task 3: Rust 侧应用数据目录与 todos 路径

**文件:**
- 修改: `src-tauri/src/lib.rs`（或 `main.rs`，以模板入口为准）
- 新建: `src-tauri/src/storage.rs`（或在同文件内实现）

**Step 1: 实现获取 todos.json 路径**

- 使用 Tauri 的 `app_handle.path().app_data_dir()` 得到应用数据目录。
- 确保目录存在（`std::fs::create_dir_all`）。
- 返回该目录下 `todos.json` 的路径（`PathBuf`）。

**Step 2: 单元测试（可选）**

在 `storage.rs` 或 `lib.rs` 中为「路径拼接」写简单测试（如临时目录下路径正确）。

**Step 3: 提交**

```bash
git add src-tauri/src/storage.rs src-tauri/src/lib.rs
git commit -m "feat(storage): 应用数据目录与 todos.json 路径"
```

---

### Task 4: Rust 命令 load_todos

**文件:**
- 修改: `src-tauri/src/lib.rs`（注册命令）
- 修改: `src-tauri/src/storage.rs` 或同模块

**Step 1: 定义 Todo 结构体**

与设计一致：`id: String`, `text: String`, `done: bool`, `order: u32`；加 `serde` 序列化。

**Step 2: 实现 load_todos**

- 读 `todos.json`；若文件不存在，返回 `Vec::new()`。
- 若解析失败（无效 JSON），返回 `Vec::new()`，不打 panic。
- 对每条缺字段做兼容：缺 `id` 则生成 UUID，缺 `text` 用 `""`，缺 `done` 用 `false`，缺 `order` 用下标。

**Step 3: 在 Tauri 中注册命令**

在 `run()` 或 builder 中注册 `load_todos`，供前端 `invoke('load_todos')` 调用。

**Step 4: 提交**

```bash
git add src-tauri/src/
git commit -m "feat(backend): load_todos 命令与 Todo 结构"
```

---

### Task 5: Rust 命令 save_todos

**文件:**
- 修改: `src-tauri/src/lib.rs`、`src-tauri/src/storage.rs`（或同模块）

**Step 1: 实现 save_todos**

- 入参：完整 `Vec<Todo>`。
- 写入 `todos.json`（先确保目录存在）；写失败时返回 `Result::Err`，前端据此提示「保存失败，请重试」。

**Step 2: 注册命令**

在 Tauri 中注册 `save_todos`。

**Step 3: 提交**

```bash
git add src-tauri/src/
git commit -m "feat(backend): save_todos 命令"
```

---

### Task 6: window.json 与窗口位置、置顶偏好

**文件:**
- 修改: `src-tauri/src/storage.rs`（或新建 `window_config.rs`）
- 修改: `src-tauri/src/lib.rs`

**Step 1: 定义窗口配置结构**

例如：`{ x: i32, y: i32, always_on_top: bool }`，对应 `window.json`。

**Step 2: 实现读/写 window.json**

- 路径：`app_data_dir() / "window.json"`。
- 读：文件不存在或解析失败时返回默认值（如屏幕居中或固定坐标，`always_on_top: true`）。
- 写：在窗口移动或置顶切换时由后续任务调用。

**Step 3: 提交**

```bash
git add src-tauri/src/
git commit -m "feat(storage): window.json 读写"
```

---

### Task 7: 启动时应用窗口位置与置顶

**文件:**
- 修改: `src-tauri/src/lib.rs`（或 `main.rs`）

**Step 1: 窗口创建后读取 window.json**

在 `run()` 或窗口 builder 的 `on_window_event`/创建完成回调中：读 `window.json`，若有有效 `x,y` 且通过简单边界检查（窗口至少部分在屏幕内），则 `set_position`；并 `set_always_on_top(window_config.always_on_top)`。

**Step 2: 提交**

```bash
git add src-tauri/src/
git commit -m "feat(窗口): 启动时恢复位置与置顶状态"
```

---

### Task 8: Rust 命令 set_always_on_top 与持久化

**文件:**
- 修改: `src-tauri/src/lib.rs`

**Step 1: 实现 set_always_on_top(enabled: bool)**

- 获取当前主窗口，调用 Tauri 的 `set_always_on_top(enabled)`。
- 将当前窗口位置（从窗口 API 读取）与新的 `always_on_top` 写回 `window.json`。

**Step 2: 注册命令**

前端置顶按钮将调用 `invoke('set_always_on_top', { body: { enabled } })`。

**Step 3: 提交**

```bash
git add src-tauri/src/
git commit -m "feat(窗口): set_always_on_top 命令并持久化"
```

---

### Task 9: 窗口拖动（前端触发）

**文件:**
- 修改: `src-tauri/src/lib.rs`
- 参考: Tauri 2 文档 Window 相关 API

**Step 1: 暴露 start_dragging 或等效能力**

Tauri 2 若提供 `window.start_dragging()` 或类似，在 Rust 中注册命令 `start_dragging`，内部获取当前窗口并调用。若无，则提供 `set_position(x, y)` 由前端在拖动区域 mousedown/mousemove/mouseup 计算偏移后调用（次选）。

**Step 2: 注册命令**

前端在「标题栏」区域 mousedown 时调用 `invoke('start_dragging')`。

**Step 3: 提交**

```bash
git add src-tauri/src/
git commit -m "feat(窗口): 支持无边框窗口拖动"
```

---

### Task 10: 窗口移动后写回 window.json

**文件:**
- 修改: `src-tauri/src/lib.rs`

**Step 1: 监听窗口位置变化**

在 Tauri 中监听主窗口的 position 变化（或拖动结束事件），在变化时读取当前 `x,y` 与当前 `always_on_top`，写回 `window.json`。注意防抖，避免拖动过程中频繁写文件。

**Step 2: 提交**

```bash
git add src-tauri/src/
git commit -m "feat(窗口): 移动后保存位置到 window.json"
```

---

### Task 11: Windows 开机启动（启动项）

**文件:**
- 修改: `src-tauri/src/lib.rs`（或新建 `src-tauri/src/autostart.rs`）

**Step 1: 实现启用/禁用开机启动**

- Windows：在 `%AppData%\Microsoft\Windows\Start Menu\Programs\Startup` 创建快捷方式（.lnk）指向当前 exe；或写 `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`。需获取已安装 exe 路径（开发时可为 dev 路径）。
- 提供命令如 `set_autostart(enabled: bool)`；禁用时删除快捷方式或注册表项。

**Step 2: 默认启用**

应用首次运行或安装后，默认调用 `set_autostart(true)`（可选，与设计「默认开机启动」一致）。

**Step 3: 提交**

```bash
git add src-tauri/src/
git commit -m "feat(backend): Windows 开机启动开关"
```

---

### Task 12: 前端 HTML 结构（标题栏 + 列表 + 添加区）

**文件:**
- 修改: `src/index.html`（或模板生成的入口 HTML）

**Step 1: 结构**

- 顶部：一块「标题栏」区域（含拖动用 data 属性或 class，如 `data-tauri-drag-region`；以及置顶按钮）。
- 中间：待办列表容器（ul/div，每条由 JS 动态渲染）。
- 底部：输入框 +「添加」按钮（或仅回车提交）。

**Step 2: 提交**

```bash
git add src/index.html
git commit -m "feat(ui): 标题栏、列表容器、添加区结构"
```

---

### Task 13: 前端样式（半透明、固定宽高、列表项、长文案换行）

**文件:**
- 修改: `src/style.css`（或 `src/assets/*.css`）

**Step 1: 全局与窗口**

- body 背景：半透明，如 `rgba(255,255,255,0.92)` 或深色等效；与 tauri 的 `transparent: true` 配合。
- 根容器宽高与窗口一致（320×400），overflow 合理（列表可滚动）。

**Step 2: 列表项**

- 每条：勾选框 + 文案 + 删除按钮；文案区域 `white-space: normal; word-break: break-word;` 实现**单条超长文案换行显示**。
- 已完成项：灰色 + 删除线（或仅灰）。

**Step 3: 标题栏与置顶按钮**

- 标题栏可设最小高度（如 32px），置顶按钮小图标或文字，风格极简。

**Step 4: 提交**

```bash
git add src/style.css
git commit -m "style(ui): 半透明、列表项、长文案换行、完成态样式"
```

---

### Task 14: 前端加载列表与渲染

**文件:**
- 修改: `src/main.js`（或模板入口 JS）

**Step 1: 页面加载时调用 load_todos**

- `invoke('load_todos')`，得到数组后按 `order` 排序（或保持顺序），渲染到列表容器。

**Step 2: 每条渲染**

- 勾选框（完成状态）、文案（可点击进入编辑）、删除按钮；长文案换行由 CSS 已保证。

**Step 3: 提交**

```bash
git add src/main.js
git commit -m "feat(ui): 加载并渲染待办列表"
```

---

### Task 15: 前端增删改与完成状态

**文件:**
- 修改: `src/main.js`

**Step 1: 添加**

- 输入框回车或点击「添加」：校验非空，生成 `id`（如 crypto.randomUUID()），`order` 取当前列表 length，`done: false`，追加到列表，调用 `save_todos(完整列表)`，再刷新 DOM 或重渲染列表。

**Step 2: 删除**

- 点击删除按钮：从列表中移除该项，调用 `save_todos(完整列表)`，更新 DOM。

**Step 3: 改（编辑文案）**

- 点击文案：该条变为行内输入框，保存时更新该项 `text`，调用 `save_todos`，恢复展示；取消则恢复原文案。

**Step 4: 完成状态**

- 勾选/取消勾选：更新该项 `done`，调用 `save_todos`，更新 DOM 样式（已完成灰+删除线）。

**Step 5: 提交**

```bash
git add src/main.js
git commit -m "feat(ui): 增删改与完成状态并持久化"
```

---

### Task 16: 前端拖动区与置顶按钮行为

**文件:**
- 修改: `src/main.js`
- 修改: `src/index.html` 或 `src/style.css`（若需补 data 属性或 class）

**Step 1: 标题栏 mousedown 触发拖动**

- 在标题栏（不含置顶按钮）上 mousedown 时调用 `invoke('start_dragging')`，使系统接管拖动。

**Step 2: 置顶按钮**

- 点击时获取当前是否置顶（可从上次调用结果或再读 window 状态），取反后调用 `invoke('set_always_on_top', { body: { enabled } })`，并更新按钮样式（高亮表示当前置顶）。

**Step 3: 提交**

```bash
git add src/main.js src/index.html
git commit -m "feat(ui): 标题栏拖动与置顶按钮"
```

---

### Task 17: 手动验证（Windows）

**步骤:**

1. 在 Windows 上执行 `npm run tauri build`，得到 exe。
2. 运行 exe：确认窗口半透明、无边框、固定大小、可拖动、置顶按钮切换有效、关闭再开位置与置顶恢复。
3. 增删改查、完成状态、长文案换行显示、保存失败提示（可临时改权限测试）。
4. 设置开机启动后重启电脑，确认开机后应用自动出现（或至少启动项存在）。

**提交（可选）:**

若发现 bug 可单独修后提交；本任务无需代码提交，仅验证。

---

## 执行选项

计划已保存至 `docs/plans/2025-02-24-desktop-todolist.md`。

**两种执行方式：**

1. **本会话子 agent 驱动** — 按任务拆分子 agent，每步完成后在本会话内审查，迭代快。  
2. **独立会话** — 在新会话中打开，使用 @superpowers:executing-plans 按任务批量执行并设置检查点。

你更倾向哪一种？
