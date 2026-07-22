mod sidecar;

use sidecar::{backend_call, backend_restart, backend_status, SidecarManager};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let manager = SidecarManager::new(app.handle().clone());
            app.manage(manager.clone());

            tauri::async_runtime::spawn(async move {
                if let Err(error) = manager.start().await {
                    eprintln!("Failed to start Nim sidecar: {error}");
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            backend_call,
            backend_status,
            backend_restart
        ])
        .build(tauri::generate_context!())
        .expect("error while building Tauri application");

    app.run(|app_handle, event| {
        if matches!(event, tauri::RunEvent::ExitRequested { .. }) {
            let manager = app_handle.state::<SidecarManager>().inner().clone();
            tauri::async_runtime::block_on(manager.shutdown());
        }
    });
}
