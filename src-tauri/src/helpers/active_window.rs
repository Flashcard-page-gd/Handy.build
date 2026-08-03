/// Returns the name of the currently active (foreground) application,
/// or `None` if it cannot be determined.
///
/// Platform behaviour:
///   - **Windows**: Uses `GetForegroundWindow` → `GetWindowThreadProcessId` →
///     `OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION)` → `QueryFullProcessImageNameW`.
///     Strips the `.exe` extension from the returned path so callers always see a
///     clean process name such as `"code"` or `"msedge"`.
///   - **macOS**: Reads `NSWorkspace.shared.frontmostApplication.localizedName`.
///     No Accessibility or Screen Recording permissions are required for the app name.
///   - **Linux**: Tries, in order:
///     1. `kdotool getactivewindow` / `kdotool getwindowclassname` (KDE Plasma Wayland)
///     2. `hyprctl activewindow -j` (Hyprland)
///     3. `xprop -root _NET_ACTIVE_WINDOW` + `xprop -id … WM_CLASS` (X11)
///     Falls back to `None` if none of the above succeed.
///   - **Other platforms**: Always returns `None`.

// ── Windows ──────────────────────────────────────────────────────────────────

#[cfg(target_os = "windows")]
use std::path::Path;
#[cfg(target_os = "windows")]
use windows::core::PWSTR;
#[cfg(target_os = "windows")]
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
};
#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

#[cfg(target_os = "windows")]
pub fn get_active_window_name() -> Option<String> {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0.is_null() {
            return None;
        }
        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == 0 {
            return None;
        }

        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;

        let mut buffer = [0u16; 1024];
        let mut size = buffer.len() as u32;
        let result = QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_WIN32,
            PWSTR(buffer.as_mut_ptr()),
            &mut size,
        );
        let _ = windows::Win32::Foundation::CloseHandle(handle);

        result.ok()?;

        let path_str = String::from_utf16_lossy(&buffer[..size as usize]);
        let path = Path::new(&path_str);
        let name = path.file_name()?.to_str()?;
        // Strip the .exe extension so callers see "code" instead of "code.exe"
        let clean = if name.to_lowercase().ends_with(".exe") {
            &name[..name.len() - 4]
        } else {
            name
        };
        Some(clean.to_string())
    }
}

// ── macOS ─────────────────────────────────────────────────────────────────────

#[cfg(target_os = "macos")]
use objc2_app_kit::NSWorkspace;

#[cfg(target_os = "macos")]
pub fn get_active_window_name() -> Option<String> {
    // SAFETY: sharedWorkspace and frontmostApplication are read-only queries
    // that are safe to call from any thread (no UI mutations).
    unsafe {
        let workspace = NSWorkspace::sharedWorkspace();
        let app = workspace.frontmostApplication()?;
        let name = app.localizedName()?;
        Some(name.to_string())
    }
}

// ── Linux ─────────────────────────────────────────────────────────────────────

#[cfg(target_os = "linux")]
use std::process::Command;

#[cfg(target_os = "linux")]
pub fn get_active_window_name() -> Option<String> {
    if crate::utils::is_wayland() {
        // KDE Plasma Wayland → kdotool
        if crate::utils::is_kde_plasma() {
            if let Some(name) = try_kdotool() {
                return Some(name);
            }
        }
        // Hyprland
        if let Some(name) = try_hyprctl() {
            return Some(name);
        }
        None
    } else {
        // X11
        try_xprop()
    }
}

#[cfg(target_os = "linux")]
fn try_kdotool() -> Option<String> {
    let id_out = Command::new("kdotool")
        .args(["getactivewindow"])
        .output()
        .ok()?;
    if !id_out.status.success() {
        return None;
    }
    let window_id = String::from_utf8_lossy(&id_out.stdout).trim().to_string();
    if window_id.is_empty() {
        return None;
    }
    let class_out = Command::new("kdotool")
        .args(["getwindowclassname", &window_id])
        .output()
        .ok()?;
    if !class_out.status.success() {
        return None;
    }
    let class_name = String::from_utf8_lossy(&class_out.stdout).trim().to_string();
    if class_name.is_empty() {
        None
    } else {
        Some(class_name)
    }
}

#[cfg(target_os = "linux")]
fn try_hyprctl() -> Option<String> {
    let out = Command::new("hyprctl")
        .args(["activewindow", "-j"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).ok()?;
    let class_name = json.get("class")?.as_str()?;
    if class_name.is_empty() {
        None
    } else {
        Some(class_name.to_string())
    }
}

#[cfg(target_os = "linux")]
fn try_xprop() -> Option<String> {
    let root_out = Command::new("xprop")
        .args(["-root", "-notype", "_NET_ACTIVE_WINDOW"])
        .output()
        .ok()?;
    if !root_out.status.success() {
        return None;
    }
    let root_str = String::from_utf8_lossy(&root_out.stdout);
    let window_id = root_str.split('=').last()?.trim();
    if window_id.is_empty() || window_id == "0x0" {
        return None;
    }

    let class_out = Command::new("xprop")
        .args(["-id", window_id, "WM_CLASS"])
        .output()
        .ok()?;
    if !class_out.status.success() {
        return None;
    }
    let class_str = String::from_utf8_lossy(&class_out.stdout);
    // WM_CLASS output: `WM_CLASS = "instance", "ClassName"`
    // We take the first quoted value (instance name)
    let val_part = class_str.split('=').last()?;
    let first = val_part.split(',').next()?;
    let clean = first.replace('"', "").trim().to_string();
    if clean.is_empty() {
        None
    } else {
        Some(clean)
    }
}

// ── Fallback ──────────────────────────────────────────────────────────────────

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
pub fn get_active_window_name() -> Option<String> {
    None
}
