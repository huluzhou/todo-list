//! Windows 开机启动：通过 HKCU\...\Run 注册表添加/移除启动项。
//! 仅编译于 Windows；非 Windows 由 lib 层返回「仅支持 Windows」。

#[cfg(windows)]
/// 规范化路径字符串，去除首尾引号和空白，用于比较
fn normalize_path(path: &str) -> String {
    path.trim()
        .trim_start_matches('"')
        .trim_end_matches('"')
        .trim()
        .to_string()
}

/// 启用或禁用开机启动（仅 Windows 有效）。
/// - enabled == true：将当前 exe 路径写入 HKCU\...\Run。
/// - enabled == false：删除 Run 下对应项。
/// 使用 std::env::current_exe() 获取 exe 路径，开发与已安装环境均适用。
#[cfg(windows)]
pub fn set_autostart_impl(enabled: bool) -> Result<(), String> {
    use std::env;
    use std::io;
    use winreg::enums::KEY_SET_VALUE;
    use winreg::RegKey;

    /// 注册表中 Run 键下使用的值名（与产品名一致，便于用户识别）。
    const RUN_VALUE_NAME: &str = "desktop-todolist";

    let exe_path = env::current_exe().map_err(|e| format!("获取 exe 路径失败: {}", e))?;
    let exe_str = exe_path
        .to_str()
        .ok_or_else(|| "exe 路径含非法字符".to_string())?;
    // 路径含空格时用引号包裹，符合 Windows Run 项惯例
    let value = if exe_str.contains(' ') {
        format!("\"{}\"", exe_str)
    } else {
        exe_str.to_string()
    };

    let hkcu = RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
    let run_path = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
    let run_key = hkcu
        .open_subkey_with_flags(run_path, KEY_SET_VALUE)
        .map_err(|e| format!("打开注册表 Run 键失败: {}", e))?;

    if enabled {
        run_key
            .set_value(RUN_VALUE_NAME, &value)
            .map_err(|e: io::Error| format!("写入开机启动项失败: {}", e))?;
    } else {
        let _ = run_key.delete_value(RUN_VALUE_NAME);
        // 若本不存在则 delete_value 返回 Err，忽略即可
    }
    Ok(())
}

/// 查询当前是否已启用开机启动（仅 Windows：读 Run 键是否存在且值为当前 exe）。
#[cfg(windows)]
pub fn is_autostart_enabled_impl() -> Result<bool, String> {
    use std::env;
    use winreg::enums::KEY_READ;
    use winreg::RegKey;

    const RUN_VALUE_NAME: &str = "desktop-todolist";

    let exe_path = env::current_exe().map_err(|e| format!("获取 exe 路径失败: {}", e))?;
    let exe_str = exe_path
        .to_str()
        .ok_or_else(|| "exe 路径含非法字符".to_string())?;

    let hkcu = RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
    let run_path = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
    let run_key = hkcu
        .open_subkey_with_flags(run_path, KEY_READ)
        .map_err(|e| format!("打开注册表 Run 键失败: {}", e))?;

    let current: String = run_key
        .get_value(RUN_VALUE_NAME)
        .unwrap_or_default();
    
    // 规范化比较：去除引号和空白后比较
    let normalized_current = normalize_path(&current);
    let normalized_exe = normalize_path(exe_str);
    
    Ok(!normalized_current.is_empty() && normalized_current == normalized_exe)
}

#[cfg(not(windows))]
#[allow(dead_code)]
pub fn set_autostart_impl(_enabled: bool) -> Result<(), String> {
    Err("仅支持 Windows".to_string())
}

#[cfg(not(windows))]
#[allow(dead_code)]
pub fn is_autostart_enabled_impl() -> Result<bool, String> {
    Ok(false)
}
