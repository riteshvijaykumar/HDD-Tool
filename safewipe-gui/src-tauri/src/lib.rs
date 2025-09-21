use tokio::sync::Mutex;
use tauri::{State, command};
use safewipe_sdk::{SafeWipeClient, SanitizationMethod};

// Global SafeWipeClient instance (for demo, use a Mutex for thread safety)
struct AppState {
    client: Mutex<SafeWipeClient>,
}

/// Simple greeting command for testing
#[command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

/// List all available devices
#[command]
async fn list_devices(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let client = {
        let guard = state.client.lock().await;
        guard.clone()
    };
    let resp = client.list_devices().await;
    serde_json::to_value(resp).map_err(|e| e.to_string())
}

/// Start a wipe operation
#[command]
async fn start_wipe(state: State<'_, AppState>, device_id: String, method: String, passes: Option<u8>, pattern: Option<String>) -> Result<serde_json::Value, String> {
    let client = {
        let guard = state.client.lock().await;
        guard.clone()
    };
    let method_enum = match method.as_str() {
        "clear" => SanitizationMethod::Clear,
        "purge" => SanitizationMethod::Purge,
        "destroy" => SanitizationMethod::Destroy,
        _ => SanitizationMethod::Clear,
    };
    let resp = client.start_wipe(&device_id, method_enum).await;
    serde_json::to_value(resp).map_err(|e| e.to_string())
}

/// Get progress of a wipe operation
#[command]
async fn get_wipe_progress(state: State<'_, AppState>, operation_id: String) -> Result<serde_json::Value, String> {
    let client = {
        let guard = state.client.lock().await;
        guard.clone()
    };
    let resp = client.get_operation_status(&operation_id).await;
    serde_json::to_value(resp).map_err(|e| e.to_string())
}

/// Verify a device after wipe
#[command]
async fn verify_device(state: State<'_, AppState>, operation_id: String) -> Result<serde_json::Value, String> {
    let client = {
        let guard = state.client.lock().await;
        guard.clone()
    };
    let resp = client.get_operation_status(&operation_id).await; // For demo, reuse status
    serde_json::to_value(resp).map_err(|e| e.to_string())
}

/// Generate a wipe report
#[command]
async fn generate_report(state: State<'_, AppState>, operation_id: String) -> Result<serde_json::Value, String> {
    let client = {
        let guard = state.client.lock().await;
        guard.clone()
    };
    let resp = client.generate_operation_report(&operation_id).await;
    serde_json::to_value(resp).map_err(|e| e.to_string())
}

/// Get current settings/config
#[command]
async fn get_settings(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let client = {
        let guard = state.client.lock().await;
        guard.clone()
    };
    let resp = client.get_config().await;
    serde_json::to_value(resp).map_err(|e| e.to_string())
}

/// Update settings/config
#[command]
async fn set_settings(state: State<'_, AppState>, config: serde_json::Value) -> Result<serde_json::Value, String> {
    let client = {
        let guard = state.client.lock().await;
        guard.clone()
    };
    let config = serde_json::from_value(config).map_err(|e| e.to_string())?;
    let resp = client.update_config(config).await;
    match resp {
        Ok(val) => serde_json::to_value(val).map_err(|e| e.to_string()),
        Err(e) => Err(e.to_string()),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState { client: Mutex::new(SafeWipeClient::new()) })
        .invoke_handler(tauri::generate_handler![
            greet,
            list_devices,
            start_wipe,
            get_wipe_progress,
            verify_device,
            generate_report,
            get_settings,
            set_settings
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
