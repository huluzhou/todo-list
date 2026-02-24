// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

mod autostart;
mod storage;

use std::sync::mpsc;
use std::time::Duration;

use tauri::Manager;
use tauri::PhysicalPosition;
use tauri::WindowEvent;

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

/// 开始拖动主窗口（供无边框窗口标题栏 mousedown 时调用）。
/// 内部获取主窗口并调用 Tauri 的 start_dragging()，由系统接管拖动。
#[tauri::command]
fn start_dragging(app: tauri::AppHandle) -> Result<(), String> {
    let main = app
        .get_webview_window("main")
        .ok_or_else(|| "主窗口不存在".to_string())?;
    main.start_dragging().map_err(|e| e.to_string())
}

/// 设置是否开机启动（仅 Windows：写/删 HKCU\\...\\Run 注册表）。
/// 供前端 invoke('set_autostart', { body: { enabled } }) 调用；非 Windows 返回 Err("仅支持 Windows")。
#[tauri::command]
fn set_autostart(enabled: bool) -> Result<(), String> {
    autostart::set_autostart_impl(enabled)
}

/// 设置主窗口是否始终置顶，并将当前窗口位置与新的 always_on_top 写回 window.json。
/// 供前端 invoke('set_always_on_top', { body: { enabled } }) 调用。
#[tauri::command]
fn set_always_on_top(app: tauri::AppHandle, enabled: bool) -> Result<(), String> {
    let main = app
        .get_webview_window("main")
        .ok_or_else(|| "主窗口不存在".to_string())?;
    main.set_always_on_top(enabled).map_err(|e| e.to_string())?;

    // 从窗口 API 读取当前位置，与新的 always_on_top 一并写回 window.json
    let (x, y) = main
        .outer_position()
        .map(|p| (p.x, p.y))
        .unwrap_or_else(|_| {
            // 读取失败时使用已保存的配置或默认值
            storage::load_window_config(&app)
                .map(|c| (c.x, c.y))
                .unwrap_or((100, 100))
        });
    let config = storage::WindowConfig {
        x,
        y,
        always_on_top: enabled,
    };
    storage::save_window_config(&app, &config)?;
    Ok(())
}

/// 简单边界检查：窗口 (x, y, width, height) 是否至少部分在给定显示器范围内。
/// 若无法获取显示器则视为通过（不校验）。
fn is_position_valid_on_monitor(
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    mon_pos: (i32, i32),
    mon_size: (u32, u32),
) -> bool {
    let (mx, my) = mon_pos;
    let (mw, mh) = mon_size;
    let x_overlap = x < (mx + mw as i32) && (x + width as i32) > mx;
    let y_overlap = y < (my + mh as i32) && (y + height as i32) > my;
    x_overlap && y_overlap
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, load_todos, save_todos, start_dragging, set_autostart, set_always_on_top])
        .setup(|app| {
            // 启动时从 window.json 恢复窗口位置与置顶状态
            let config = match storage::load_window_config(app) {
                Ok(c) => c,
                Err(_) => return Ok(()),
            };

            let main = match app.get_webview_window("main") {
                Some(w) => w,
                None => return Ok(()),
            };

            // 置顶状态
            let _ = main.set_always_on_top(config.always_on_top);

            // 位置：有效 x,y 且通过简单边界检查则设置
            let width = 320u32;
            let height = 400u32;
            let valid = if let Ok(Some(mon)) = main.primary_monitor() {
                let pos = mon.position();
                let size = mon.size();
                is_position_valid_on_monitor(
                    config.x,
                    config.y,
                    width,
                    height,
                    (pos.x as i32, pos.y as i32),
                    (size.width, size.height),
                )
            } else {
                // 无法获取显示器时做数值范围检查，避免明显越界
                config.x >= -32768
                    && config.x <= 32767
                    && config.y >= -32768
                    && config.y <= 32767
            };

            if valid {
                let _ = main.set_position(PhysicalPosition::new(config.x as f64, config.y as f64));
            }

            // 监听主窗口位置变化（含拖动结束），防抖后写回 window.json
            let (tx, rx) = mpsc::channel();
            let app_handle = app.handle().clone();
            main.on_window_event(move |event| {
                if let WindowEvent::Moved(_) = event {
                    let _ = tx.send(());
                }
            });
            std::thread::spawn(move || {
                while rx.recv().is_ok() {
                    // 防抖：等待 300ms，期间若有新的 Moved 则继续等待
                    loop {
                        std::thread::sleep(Duration::from_millis(300));
                        if rx.try_recv().is_err() {
                            break;
                        }
                    }
                    let handle = app_handle.clone();
                    let _ = app_handle.run_on_main_thread(move || {
                        if let Some(m) = handle.get_webview_window("main") {
                            let (x, y) = m.outer_position().map(|p| (p.x, p.y)).unwrap_or((100, 100));
                            let always_on_top = m.is_always_on_top().unwrap_or(true);
                            let config = storage::WindowConfig {
                                x,
                                y,
                                always_on_top,
                            };
                            let _ = storage::save_window_config(&handle, &config);
                        }
                    });
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
