// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

mod storage;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

/// 加载待办列表，供前端 invoke('load_todos') 调用。
#[tauri::command]
fn load_todos(app: tauri::AppHandle) -> Result<Vec<storage::Todo>, String> {
    storage::load_todos(app)
}

/// 保存完整待办列表到 todos.json，供前端 invoke('save_todos', { body: { todos } }) 调用。
/// 写失败时返回 Err，前端可提示「保存失败，请重试」。
#[tauri::command]
fn save_todos(app: tauri::AppHandle, todos: Vec<storage::Todo>) -> Result<(), String> {
    storage::save_todos(&app, &todos)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, load_todos, save_todos])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
