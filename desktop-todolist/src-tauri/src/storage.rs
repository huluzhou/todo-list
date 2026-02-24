//! 应用数据目录与 todos.json / window.json 路径。
//! 使用 Tauri 2 的 PathResolver（app.path().app_data_dir()）解析应用数据目录。

use std::path::{Path, PathBuf};

use tauri::{Manager, Runtime};

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
