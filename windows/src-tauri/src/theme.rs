// 窗口底色跟着主题走。
//
// 为什么需要这一层：WebView2 从进程起来到画出第一帧要 1.9–3.3 秒，这段时间窗口
// 显示的是 tauri.conf.json 里写死的 backgroundColor（浅色）。深色用户就会先看到
// 一块白板再变黑，非常刺眼。窗口在 setup 之前就被创建好了，所以在 setup 里把
// 底色和系统主题一起刷成对应的值，第一帧出来就是对的。
//
// 手动切主题时前端会调 set_app_theme，走同一个函数，保证「切完立刻切、下次打开也记得」。

use tauri::window::Color;
use tauri::{AppHandle, Manager, WebviewWindow};

/// 浅色底：和 style.css 的 --window 一致（rgb(251,250,248)）
pub const LIGHT_BG: Color = Color(251, 250, 248, 255);
/// 深色底：和 style.css 深色 --window 一致（rgb(27,27,29)）
pub const DARK_BG: Color = Color(27, 27, 29, 255);

/// 系统是不是深色模式：读注册表。读不到就当浅色（中文 Windows 默认都是浅色）
pub fn system_dark() -> bool {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;
    RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize")
        .and_then(|k| k.get_value::<u32, _>("AppsUseLightTheme"))
        .map(|light| light == 0)
        .unwrap_or(false)
}

/// 三种偏好（light / dark / system）折算成实际深浅
pub fn is_dark(pref: &str) -> bool {
    match pref {
        "dark" => true,
        "light" => false,
        _ => system_dark(),
    }
}

pub fn background(pref: &str) -> Color {
    if is_dark(pref) {
        DARK_BG
    } else {
        LIGHT_BG
    }
}

pub fn preference(app: &AppHandle) -> String {
    app.state::<crate::AppState>().settings.lock().unwrap().theme.clone()
}

pub fn normalize(pref: &str) -> &str {
    match pref {
        "light" | "dark" => pref,
        _ => "system",
    }
}

pub fn native(pref: &str) -> Option<tauri::Theme> {
    match pref {
        "light" => Some(tauri::Theme::Light),
        "dark" => Some(tauri::Theme::Dark),
        _ => None,
    }
}

pub fn init(pref: &str) -> String {
    format!("window.__sdTheme = '{}';", normalize(pref))
}

pub fn apply_view(window: &WebviewWindow, pref: &str) {
    let _ = window.eval(format!("{}window.__sdSetTheme?.(window.__sdTheme);", init(pref)));
}

/// 用户手动切主题时用：标题栏和窗口底色一起改（下次启动由建窗口时传入的值接管）
pub fn apply(window: &WebviewWindow, pref: &str) {
    let _ = window.set_theme(native(pref));
    // Win11 保留透明底色，由系统统一裁剪四角；Win10 继续使用不透明底色。
    if legacy_frame() {
        let _ = window.set_background_color(Some(background(pref)));
    }
}

/// Win11 从 build 22000 开始；不能用浏览器的 Windows NT 10.0 区分。
pub fn legacy_frame() -> bool {
    use winreg::{enums::HKEY_LOCAL_MACHINE, RegKey};
    let build = RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion")
        .and_then(|key| key.get_value::<String, _>("CurrentBuildNumber"))
        .ok().and_then(|v| v.parse::<u32>().ok());
    build.map_or(true, |n| n < 22000)
}

pub fn frame_init() -> String {
    format!("window.__sdWindowFrame = '{}';", if legacy_frame() { "square" } else { "rounded" })
}

/// Capture a single left click for the screen picker on the installing thread.
#[tauri::command]
pub async fn pick_screen_color() -> Result<Option<String>, String> {
    tauri::async_runtime::spawn_blocking(pick_screen_color_blocking)
        .await
        .map_err(|_| "取色线程未能完成".to_string())?
}

fn pick_screen_color_blocking() -> Result<Option<String>, String> {
    use std::{cell::Cell, ptr::null_mut, sync::Mutex, thread, time::{Duration, Instant}};
    use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_ESCAPE};
    use windows_sys::Win32::{
        Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM},
        Graphics::{Dwm::DwmFlush, Gdi::{GetDC, GetPixel, GetStockObject, ReleaseDC, BLACK_BRUSH}},
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::*,
    };

    static PICKER: Mutex<()> = Mutex::new(());
    let _active = PICKER.try_lock().map_err(|_| "取色已在进行".to_string())?;
    thread_local! {
        static CLICK: Cell<Option<POINT>> = const { Cell::new(None) };
    }
    unsafe extern "system" fn window_proc(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
        if msg == WM_LBUTTONUP {
            let mut point = POINT { x: 0, y: 0 };
            if GetCursorPos(&mut point) != 0 { CLICK.set(Some(point)); }
            return 0;
        }
        DefWindowProcW(hwnd, msg, w, l)
    }

    // Monochrome cursor masks. The pipette tip is at (5, 26).
    let shape = |x: i32, y: i32| {
        let axis = x - y;
        let across = (x + y - 31).abs();
        (-21..=7).contains(&axis) && across <= 2
            || (3..=8).contains(&axis) && across <= 5
            || (8..=21).contains(&axis) && across <= 4
    };
    let mut and = [255u8; 128];
    let mut xor = [0u8; 128];
    for y in 0..32 {
        for x in 0..32 {
            let inside = shape(x, y);
            let edge = !inside && (-1..=1).any(|dy| (-1..=1).any(|dx| shape(x + dx, y + dy)));
            if inside || edge {
                let index = (y * 4 + x / 8) as usize;
                let bit = 0x80 >> (x % 8);
                and[index] &= !bit;
                if edge { xor[index] |= bit; }
            }
        }
    }

    let class: Vec<u16> = "ShandianScreenColorPicker\0".encode_utf16().collect();
    let instance = unsafe { GetModuleHandleW(std::ptr::null()) };
    let cursor = unsafe { CreateCursor(instance, 5, 26, 32, 32, and.as_ptr().cast(), xor.as_ptr().cast()) };
    if cursor.is_null() { return Err("无法创建吸管光标".into()); }
    let registered = unsafe {
        RegisterClassW(&WNDCLASSW {
            lpfnWndProc: Some(window_proc),
            hInstance: instance,
            hCursor: cursor,
            hbrBackground: GetStockObject(BLACK_BRUSH),
            lpszClassName: class.as_ptr(),
            ..Default::default()
        })
    };
    if registered == 0 {
        unsafe { DestroyCursor(cursor); }
        return Err("无法启动 Windows 取色".into());
    }

    let overlay = unsafe {
        CreateWindowExW(
            WS_EX_TOPMOST | WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
            class.as_ptr(), std::ptr::null(), WS_POPUP,
            GetSystemMetrics(SM_XVIRTUALSCREEN), GetSystemMetrics(SM_YVIRTUALSCREEN),
            GetSystemMetrics(SM_CXVIRTUALSCREEN), GetSystemMetrics(SM_CYVIRTUALSCREEN),
            null_mut(), null_mut(), instance, std::ptr::null(),
        )
    };
    let point = if !overlay.is_null() && unsafe { SetLayeredWindowAttributes(overlay, 0, 1, LWA_ALPHA) } != 0 {
        CLICK.set(None);
        unsafe {
            ShowWindow(overlay, SW_SHOWNOACTIVATE);
            SetCursor(cursor);
        }
        let until = Instant::now() + Duration::from_secs(15);
        let mut msg = MSG::default();
        loop {
            unsafe {
                while PeekMessageW(&mut msg, overlay, 0, 0, PM_REMOVE) != 0 {
                    DispatchMessageW(&msg);
                }
            }
            if let Some(point) = CLICK.get() { break Ok(Some(point)); }
            if Instant::now() >= until || unsafe { GetAsyncKeyState(VK_ESCAPE.0.into()) } < 0 {
                break Ok(None);
            }
            thread::sleep(Duration::from_millis(8));
        }
    } else {
        Err("无法显示取色层")
    };
    unsafe {
        if !overlay.is_null() { DestroyWindow(overlay); }
        UnregisterClassW(class.as_ptr(), instance);
        if GetCursor() == cursor { SetCursor(LoadCursorW(null_mut(), IDC_ARROW)); }
        DestroyCursor(cursor);
    }

    let Some(point) = point? else { return Ok(None) };
    let color = unsafe {
        // Let the compositor remove the picker before sampling the underlying desktop.
        DwmFlush();
        let dc = GetDC(null_mut());
        if dc.is_null() { return Err("无法读取屏幕颜色".into()); }
        let color = GetPixel(dc, point.x, point.y);
        ReleaseDC(null_mut(), dc);
        color
    };
    if color == u32::MAX { return Err("无法读取屏幕颜色".into()); }
    Ok(Some(format!("#{:02X}{:02X}{:02X}", color & 255, (color >> 8) & 255, (color >> 16) & 255)))
}
