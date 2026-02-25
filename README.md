# todo-list

Windows 桌面极简待办应用（Tauri 2 + Vanilla JS），半透明无边框小窗、置顶可切换。

## 开发与测试

- **应用目录**：`desktop-todolist/`
- **CI**：推送到 `main` 或提 PR 时，GitHub Actions 在 Ubuntu 与 Windows 上跑测试并构建，见 [.github/workflows/ci.yml](.github/workflows/ci.yml)

## 本地运行（Windows）

```bash
cd desktop-todolist
npm install
npm run tauri dev
```

## 构建

```bash
cd desktop-todolist
npm run tauri build
```

产物在 `desktop-todolist/src-tauri/target/release/bundle/`。
