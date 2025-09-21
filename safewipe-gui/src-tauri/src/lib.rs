use safewipe_sdk::{WipeResult, SanitizationMethod};
use tauri::command;

#[command]
pub async fn tauri_wipe_device(device_path: String, method: String) -> Result<WipeResult, String> {
    // Parse method string to SanitizationMethod
    let method_enum = match method.as_str() {
        "Clear" => SanitizationMethod::Clear,
        "Purge" => SanitizationMethod::Purge,
        "Destroy" => SanitizationMethod::Destroy,
        _ => return Err("Invalid sanitization method".to_string()),
    };
    safewipe_sdk::wipe_device(&device_path, method_enum).await.map_err(|e| e.to_string())
}

#[command]
pub fn tauri_generate_wipe_report(device_path: String) -> Result<String, String> {
    safewipe_sdk::generate_wipe_report(&device_path)
        .map(|report| serde_json::to_string(&report).unwrap())
        .map_err(|e| e.to_string())
}

#[command]
pub fn tauri_verify_device_wipe(device_path: String) -> Result<bool, String> {
    safewipe_sdk::verify_device_wipe(&device_path).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }
      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      tauri_wipe_device,
      tauri_generate_wipe_report,
      tauri_verify_device_wipe
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
