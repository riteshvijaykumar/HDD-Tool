use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{command, Manager, State};
use tokio::sync::RwLock;
use safewipe_sdk::{
    SafeWipeClient, SafeWipeConfig, ApiResponse, DeviceInfo, WipeOperation, SystemStats,
    SanitizationMethod, WipeProgress,
};
use tokio::sync::broadcast;
use tauri_plugin_shell::ShellExt;

// Application state
pub struct AppState {
    client: Arc<SafeWipeClient>,
    progress_receiver: Arc<RwLock<Option<broadcast::Receiver<WipeProgress>>>>,
}

impl AppState {
    pub fn new() -> Self {
        let client = Arc::new(SafeWipeClient::new());
        Self {
            client,
            progress_receiver: Arc::new(RwLock::new(None)),
        }
    }

    pub fn with_config(config: SafeWipeConfig) -> Self {
        let client = Arc::new(SafeWipeClient::new().with_config(config));
        Self {
            client,
            progress_receiver: Arc::new(RwLock::new(None)),
        }
    }
}

// Tauri Commands

#[command]
async fn get_config(state: State<'_, AppState>) -> Result<SafeWipeConfig, String> {
    Ok(state.client.get_config().await)
}

#[command]
async fn update_config(
    config: SafeWipeConfig,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state.client.update_config(config).await
        .map_err(|e| e.to_string())
}

#[command]
async fn list_devices(state: State<'_, AppState>) -> Result<ApiResponse<Vec<DeviceInfo>>, String> {
    Ok(state.client.list_devices().await)
}

#[command]
async fn get_device_info(
    device_id: String,
    state: State<'_, AppState>,
) -> Result<ApiResponse<DeviceInfo>, String> {
    Ok(state.client.get_device_info(&device_id).await)
}

#[command]
async fn start_wipe(
    device_id: String,
    method: String,
    state: State<'_, AppState>,
) -> Result<ApiResponse<String>, String> {
    let sanitization_method = match method.as_str() {
        "clear" => SanitizationMethod::Clear,
        "purge" => SanitizationMethod::Purge,
        "destroy" => SanitizationMethod::Destroy,
        _ => return Ok(ApiResponse::error("Invalid sanitization method".to_string())),
    };

    // Set up progress receiver if not already done
    {
        let mut receiver_guard = state.progress_receiver.write().await;
        if receiver_guard.is_none() {
            *receiver_guard = Some(state.client.subscribe_progress());
        }
    }

    Ok(state.client.start_wipe(&device_id, sanitization_method).await)
}

#[command]
async fn get_operation_status(
    operation_id: String,
    state: State<'_, AppState>,
) -> Result<ApiResponse<WipeOperation>, String> {
    Ok(state.client.get_operation_status(&operation_id).await)
}

#[command]
async fn list_operations(state: State<'_, AppState>) -> Result<ApiResponse<Vec<WipeOperation>>, String> {
    Ok(state.client.list_operations().await)
}

#[command]
async fn cancel_operation(
    operation_id: String,
    state: State<'_, AppState>,
) -> Result<ApiResponse<bool>, String> {
    Ok(state.client.cancel_operation(&operation_id).await)
}

#[command]
async fn get_system_stats(state: State<'_, AppState>) -> Result<ApiResponse<SystemStats>, String> {
    Ok(state.client.get_system_stats().await)
}

#[command]
async fn verify_device(
    device_path: String,
    state: State<'_, AppState>,
) -> Result<ApiResponse<bool>, String> {
    Ok(state.client.verify_device(&device_path).await)
}

#[command]
async fn get_progress_updates(state: State<'_, AppState>) -> Result<Vec<WipeProgress>, String> {
    let mut receiver_guard = state.progress_receiver.write().await;

    if let Some(receiver) = receiver_guard.as_mut() {
        let mut updates = Vec::new();

        // Try to receive up to 10 progress updates without blocking
        for _ in 0..10 {
            match receiver.try_recv() {
                Ok(progress) => updates.push(progress),
                Err(broadcast::error::TryRecvError::Empty) => break,
                Err(broadcast::error::TryRecvError::Lagged(_)) => {
                    // Skip lagged messages and continue
                    continue;
                }
                Err(broadcast::error::TryRecvError::Closed) => {
                    // Channel closed, reset receiver
                    *receiver_guard = Some(state.client.subscribe_progress());
                    break;
                }
            }
        }

        Ok(updates)
    } else {
        Ok(Vec::new())
    }
}

// Additional utility commands



#[command]
async fn export_report(
    operation_id: String,
    format: String,
    state: State<'_, AppState>,
) -> Result<ApiResponse<String>, String> {
    match state.client.generate_operation_report(&operation_id).await {
        response if response.success => {
            // In a real implementation, you'd save the report to a file
            // and return the file path
            Ok(ApiResponse::success(format!("Report exported as {}", format)))
        },
        error_response => Ok(api_response_report_to_string(error_response)),
    }
}

// Type conversion helpers
fn api_response_report_to_string(response: ApiResponse<safewipe_sdk::Report>) -> ApiResponse<String> {
    ApiResponse {
        success: response.success,
        data: response.data.map(|_| "Report generated".to_string()),
        error: response.error,
        timestamp: response.timestamp,
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Create application state with default config
    let app_state = AppState::new();

    tauri::Builder::default()
        .manage(app_state)
        .plugin(tauri_plugin_shell::init())
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
            get_config,
            update_config,
            list_devices,
            get_device_info,
            start_wipe,
            get_operation_status,
            list_operations,
            cancel_operation,
            get_system_stats,
            verify_device,
            get_progress_updates,
            export_report
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
