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

/// 应用数据目录下 `todos.json` 的文件名。
pub const TODOS_FILENAME: &str = "todos.json";

/// 给定应用数据目录，返回其中的 `todos.json` 路径（仅做路径拼接，不创建目录）。
#[inline]
pub fn todos_json_path_in_dir(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join(TODOS_FILENAME)
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
}
