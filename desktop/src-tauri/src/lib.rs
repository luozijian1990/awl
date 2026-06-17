#![cfg_attr(test, allow(dead_code))]

mod commands;
mod state;

#[cfg(not(test))]
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, WindowEvent,
};

/// 显示并聚焦主窗口（从托盘 / dock 恢复）。
#[cfg(not(test))]
fn show_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

#[cfg(not(test))]
pub fn run() {
    let app = tauri::Builder::default()
        // 开机自启：macOS LaunchAgent / Windows 注册表 / Linux .desktop。
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .setup(|app| {
            // 托盘图标 + 菜单（macOS 顶部状态栏 / Windows 右下角通知区）。
            let show = MenuItem::with_id(app, "show", "显示 Worklog", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;

            TrayIconBuilder::with_id("main-tray")
                .icon(app.default_window_icon().expect("window icon").clone())
                .tooltip("Worklog")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => show_main_window(app),
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        show_main_window(tray.app_handle());
                    }
                })
                .build(app)?;
            Ok(())
        })
        // 关闭窗口默认不退出，改为隐藏到托盘。
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .manage(state::AppState::open().expect("failed to open worklog desktop state"))
        .invoke_handler(tauri::generate_handler![
            commands::list_work_entries,
            commands::create_work_entry,
            commands::update_work_entry,
            commands::confirm_work_entry,
            commands::archive_work_entry,
            commands::delete_work_entry,
            commands::list_work_calendars,
            commands::create_work_calendar,
            commands::update_work_calendar,
            commands::delete_work_calendar,
            commands::set_default_calendar,
            commands::export_report_source,
            commands::get_settings,
            commands::save_settings,
            commands::save_filters,
        ])
        .build(tauri::generate_context!())
        .expect("failed to build worklog desktop app");

    app.run(|_app_handle, _event| {
        // macOS：dock 图标被点击且无可见窗口时，恢复主窗口。
        // RunEvent::Reopen 是 macOS 专属变体，其它平台需 cfg 掉，否则无法编译。
        #[cfg(target_os = "macos")]
        if let tauri::RunEvent::Reopen { .. } = _event {
            show_main_window(_app_handle);
        }
    });
}

#[cfg(test)]
pub fn run() {}

#[cfg(test)]
mod tests;
