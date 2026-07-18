mod commands;
mod state;

use state::CosyncState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            // Resolve the app data directory for identity + database storage
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data dir");
            let managed_state = CosyncState::new(app_data_dir);

            // Initialize tracing to a log file inside the app data dir
            let log_dir = app
                .path()
                .app_log_dir()
                .expect("failed to resolve log dir");
            std::fs::create_dir_all(&log_dir).ok();
            let log_file = std::fs::File::create(log_dir.join("cosync.log")).unwrap();
            tracing_subscriber::fmt()
                .with_writer(log_file)
                .with_env_filter(
                    tracing_subscriber::EnvFilter::try_from_default_env()
                        .unwrap_or_else(|_| "cosync_core=debug,cosync_desktop=debug".into()),
                )
                .init();

            // Register the state so commands can access it via `State<'_, CosyncState>`
            app.manage(managed_state);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_device_info,
            commands::get_connection_state,
            commands::start_discovery,
            commands::stop_discovery,
            commands::pair_with_device,
            commands::unpair_device,
            commands::get_paired_devices,
            commands::get_clipboard_history,
            commands::get_device_fingerprint,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}