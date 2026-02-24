//! 应用数据目录与 todos.json / window.json 路径。
//! 使用 Tauri 2 的 PathResolver（app.path().app_data_dir()）解析应用数据目录。
//! 含 Todo 结构体与 load_todos 读取逻辑。

use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, Runtime};

/// 单条待办，与设计一致：id、文案、完成状态、排序。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Todo {
    pub id: String,
    pub text: String,
    pub done: bool,
    pub order: u32,
}

/// 反序列化时允许缺字段，用 Option + default 补全。
#[derive(Debug, Deserialize)]
struct TodoRaw {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    text: Option<String>,
    #[serde(default)]
    done: Option<bool>,
    #[serde(default)]
    order: Option<u32>,
}

/// 窗口配置：位置与置顶偏好，对应 `window.json`。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    /// 窗口左上角 x 坐标
    pub x: i32,
    /// 窗口左上角 y 坐标
    pub y: i32,
    /// 是否始终置顶
    #[serde(rename = "alwaysOnTop")]
    pub always_on_top: bool,
}

/// 反序列化时允许缺字段，用 Option + default 补全。
#[derive(Debug, Deserialize)]
struct WindowConfigRaw {
    #[serde(default)]
    x: Option<i32>,
    #[serde(default)]
    y: Option<i32>,
    #[serde(rename = "alwaysOnTop", default)]
    always_on_top: Option<bool>,
}

/// 应用数据目录下 `todos.json` 的文件名。
pub const TODOS_FILENAME: &str = "todos.json";

/// 应用数据目录下 `window.json` 的文件名。
pub const WINDOW_FILENAME: &str = "window.json";

/// 给定应用数据目录，返回其中的 `todos.json` 路径（仅做路径拼接，不创建目录）。
#[inline]
pub fn todos_json_path_in_dir(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join(TODOS_FILENAME)
}

/// 给定应用数据目录，返回其中的 `window.json` 路径（仅做路径拼接，不创建目录）。
#[inline]
pub fn window_config_path_in_dir(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join(WINDOW_FILENAME)
}

/// 返回应用数据目录下 `todos.json` 的完整路径；若目录不存在会先创建。
///
/// 使用 Tauri 2 的 `app.path().app_data_dir()` 获取应用数据目录（如 Windows 上
/// `%AppData%/<bundle_identifier>/`），然后确保该目录存在，并返回其中的 `todos.json` 路径。
///
/// # 参数
/// * `app` - 实现了 `tauri::Manager<R>` 的类型（如 `AppHandle`、`App`），用于获取 PathResolver。
///
/// # 错误
/// 当无法解析应用数据目录或创建目录失败时返回错误。
pub fn todos_json_path<M: Manager<R>, R: Runtime>(app: &M) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    std::fs::create_dir_all(&dir)?;
    Ok(todos_json_path_in_dir(&dir))
}

/// 返回应用数据目录下 `window.json` 的完整路径；若目录不存在会先创建。
///
/// 与 `todos_json_path` 同目录（即 `app.path().app_data_dir() / "window.json"`）。
///
/// # 参数
/// * `app` - 实现了 `tauri::Manager<R>` 的类型（如 `AppHandle`、`App`），用于获取 PathResolver。
///
/// # 错误
/// 当无法解析应用数据目录或创建目录失败时返回错误。
pub fn window_config_path<M: Manager<R>, R: Runtime>(app: &M) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    std::fs::create_dir_all(&dir)?;
    Ok(window_config_path_in_dir(&dir))
}

/// 从应用数据目录下的 `todos.json` 加载待办列表。
///
/// - 路径通过 `todos_json_path(&app)` 获取；若解析路径失败则返回 `Err`。
/// - 文件不存在或解析失败（无效 JSON）时返回 `Ok(Vec::new())`，不 panic、不弹窗。
/// - 对每条缺字段做兼容：缺 `id` 则生成 UUID，缺 `text` 用 `""`，缺 `done` 用 `false`，缺 `order` 用下标。
pub fn load_todos(app: AppHandle) -> Result<Vec<Todo>, String> {
    let path = todos_json_path(&app).map_err(|e| e.to_string())?;
    let contents = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(e.to_string()),
    };
    let raw_list: Vec<TodoRaw> = match serde_json::from_str(&contents) {
        Ok(v) => v,
        Err(_) => return Ok(Vec::new()),
    };
    let todos = raw_list
        .into_iter()
        .enumerate()
        .map(|(i, r)| Todo {
            id: r.id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            text: r.text.unwrap_or_default(),
            done: r.done.unwrap_or(false),
            order: r.order.unwrap_or(i as u32),
        })
        .collect();
    Ok(todos)
}

/// 将完整待办列表写入应用数据目录下的 `todos.json`。
///
/// - 路径通过 `todos_json_path(&app)` 获取（该函数保证目录存在）。
/// - 写失败时返回 `Err(String)`，供前端提示「保存失败，请重试」。
pub fn save_todos<M: Manager<R>, R: Runtime>(app: &M, todos: &[Todo]) -> Result<(), String> {
    let path = todos_json_path(app).map_err(|e| e.to_string())?;
    let json = serde_json::to_string_pretty(todos).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())?;
    Ok(())
}

/// 默认窗口配置：文件不存在或解析失败时使用。
fn default_window_config() -> WindowConfig {
    WindowConfig {
        x: 100,
        y: 100,
        always_on_top: true,
    }
}

/// 从应用数据目录下的 `window.json` 加载窗口配置。
///
/// - 路径通过 `window_config_path(&app)` 获取；若解析路径失败则返回 `Err`。
/// - 文件不存在或解析失败（无效 JSON）时返回默认值（x: 100, y: 100, always_on_top: true）。
/// - 缺字段时用默认值补全。
pub fn load_window_config(app: AppHandle) -> Result<WindowConfig, String> {
    let path = window_config_path(&app).map_err(|e| e.to_string())?;
    let contents = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(default_window_config()),
        Err(e) => return Err(e.to_string()),
    };
    let raw: WindowConfigRaw = match serde_json::from_str(&contents) {
        Ok(r) => r,
        Err(_) => return Ok(default_window_config()),
    };
    Ok(WindowConfig {
        x: raw.x.unwrap_or(100),
        y: raw.y.unwrap_or(100),
        always_on_top: raw.always_on_top.unwrap_or(true),
    })
}

/// 将窗口配置写入应用数据目录下的 `window.json`。
///
/// - 路径通过 `window_config_path(&app)` 获取（该函数保证目录存在）。
/// - 供后续 Task 7/8/10 在窗口移动或置顶切换时调用。
pub fn save_window_config<M: Manager<R>, R: Runtime>(app: &M, config: &WindowConfig) -> Result<(), String> {
    let path = window_config_path(app).map_err(|e| e.to_string())?;
    let json = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn todos_json_path_in_dir_ends_with_todos_json() {
        let tmp = std::env::temp_dir().join("test-todolist-storage");
        let path = todos_json_path_in_dir(&tmp);
        assert!(
            path.ends_with(TODOS_FILENAME),
            "路径应以 todos.json 结尾，得到: {:?}",
            path
        );
        assert_eq!(path.file_name().unwrap(), TODOS_FILENAME);
    }

    #[test]
    fn window_config_path_in_dir_ends_with_window_json() {
        let tmp = std::env::temp_dir().join("test-todolist-storage");
        let path = window_config_path_in_dir(&tmp);
        assert!(
            path.ends_with(WINDOW_FILENAME),
            "路径应以 window.json 结尾，得到: {:?}",
            path
        );
        assert_eq!(path.file_name().unwrap(), WINDOW_FILENAME);
    }
}
