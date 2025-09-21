//! FFI (C ABI) wrappers for SafeWipe SDK methods
// This file exposes all major SafeWipeClient methods as extern "C" functions for use from Kotlin/JNI, C, etc.
// All functions use C-compatible types (C strings, ints, etc.) and return JSON strings for complex data.

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_uint};
use safewipe_engine::wipe::SanitizationMethod;
use crate::{SafeWipeClient, ApiResponse};

/// Helper: Convert a Rust String to a C string pointer (caller must free with free_cstring)
fn to_cstring(s: String) -> *mut c_char {
    CString::new(s).unwrap().into_raw()
}

/// Free a C string allocated by this library
#[no_mangle]
pub extern "C" fn free_cstring(s: *mut c_char) {
    if s.is_null() { return; }
    unsafe { CString::from_raw(s); }
}

/// List all devices (returns JSON array)
#[no_mangle]
pub extern "C" fn ffi_list_devices_json() -> *mut c_char {
    let client = SafeWipeClient::new();
    let resp = tokio::runtime::Runtime::new().unwrap().block_on(client.list_devices());
    let json = serde_json::to_string(&resp).unwrap_or_else(|_| "{}".to_string());
    to_cstring(json)
}

/// List all sanitization methods (returns JSON array)
#[no_mangle]
pub extern "C" fn ffi_list_sanitization_methods_json() -> *mut c_char {
    let client = SafeWipeClient::new();
    let resp = client.list_sanitization_methods();
    let json = serde_json::to_string(&resp).unwrap_or_else(|_| "{}".to_string());
    to_cstring(json)
}

/// Start a wipe operation (device_id: C string, method: 0=Clear, 1=Purge, 2=Destroy)
/// Returns JSON ApiResponse<String> (operation id)
#[no_mangle]
pub extern "C" fn ffi_start_wipe(device_id: *const c_char, method: c_uint) -> *mut c_char {
    let device_id = unsafe { CStr::from_ptr(device_id).to_string_lossy().to_string() };
    let method = match method {
        0 => SanitizationMethod::Clear,
        1 => SanitizationMethod::Purge,
        2 => SanitizationMethod::Destroy,
        _ => SanitizationMethod::Clear,
    };
    let client = SafeWipeClient::new();
    let resp = tokio::runtime::Runtime::new().unwrap().block_on(client.start_wipe(&device_id, method));
    let json = serde_json::to_string(&resp).unwrap_or_else(|_| "{}".to_string());
    to_cstring(json)
}

/// Verify a device (device_id: C string) -> JSON ApiResponse<bool>
#[no_mangle]
pub extern "C" fn ffi_verify_device(device_id: *const c_char) -> *mut c_char {
    let device_id = unsafe { CStr::from_ptr(device_id).to_string_lossy().to_string() };
    let client = SafeWipeClient::new();
    let resp = tokio::runtime::Runtime::new().unwrap().block_on(client.verify_device(&device_id));
    let json = serde_json::to_string(&resp).unwrap_or_else(|_| "{}".to_string());
    to_cstring(json)
}

/// Generate a report for a device (device_id: C string) -> JSON ApiResponse<Report>
#[no_mangle]
pub extern "C" fn ffi_generate_report(device_id: *const c_char) -> *mut c_char {
    let device_id = unsafe { CStr::from_ptr(device_id).to_string_lossy().to_string() };
    let client = SafeWipeClient::new();
    let resp = tokio::runtime::Runtime::new().unwrap().block_on(client.generate_report(&device_id));
    let json = serde_json::to_string(&resp).unwrap_or_else(|_| "{}".to_string());
    to_cstring(json)
}

/// Get operation status (operation_id: C string) -> JSON ApiResponse<WipeOperation>
#[no_mangle]
pub extern "C" fn ffi_get_operation_status(operation_id: *const c_char) -> *mut c_char {
    let operation_id = unsafe { CStr::from_ptr(operation_id).to_string_lossy().to_string() };
    let client = SafeWipeClient::new();
    let resp = tokio::runtime::Runtime::new().unwrap().block_on(client.get_operation_status(&operation_id));
    let json = serde_json::to_string(&resp).unwrap_or_else(|_| "{}".to_string());
    to_cstring(json)
}

/// List all operations -> JSON ApiResponse<Vec<WipeOperation>>
#[no_mangle]
pub extern "C" fn ffi_list_operations() -> *mut c_char {
    let client = SafeWipeClient::new();
    let resp = tokio::runtime::Runtime::new().unwrap().block_on(client.list_operations());
    let json = serde_json::to_string(&resp).unwrap_or_else(|_| "{}".to_string());
    to_cstring(json)
}

/// Cancel an operation (operation_id: C string) -> JSON ApiResponse<bool>
#[no_mangle]
pub extern "C" fn ffi_cancel_operation(operation_id: *const c_char) -> *mut c_char {
    let operation_id = unsafe { CStr::from_ptr(operation_id).to_string_lossy().to_string() };
    let client = SafeWipeClient::new();
    let resp = tokio::runtime::Runtime::new().unwrap().block_on(client.cancel_operation(&operation_id));
    let json = serde_json::to_string(&resp).unwrap_or_else(|_| "{}".to_string());
    to_cstring(json)
}

/// Generate a report for a completed operation (operation_id: C string) -> JSON ApiResponse<Report>
#[no_mangle]
pub extern "C" fn ffi_generate_operation_report(operation_id: *const c_char) -> *mut c_char {
    let operation_id = unsafe { CStr::from_ptr(operation_id).to_string_lossy().to_string() };
    let client = SafeWipeClient::new();
    let resp = tokio::runtime::Runtime::new().unwrap().block_on(client.generate_operation_report(&operation_id));
    let json = serde_json::to_string(&resp).unwrap_or_else(|_| "{}".to_string());
    to_cstring(json)
}

/// Get system statistics -> JSON ApiResponse<SystemStats>
#[no_mangle]
pub extern "C" fn ffi_get_system_stats() -> *mut c_char {
    let client = SafeWipeClient::new();
    let resp = tokio::runtime::Runtime::new().unwrap().block_on(client.get_system_stats());
    let json = serde_json::to_string(&resp).unwrap_or_else(|_| "{}".to_string());
    to_cstring(json)
}

// Add more FFI wrappers as needed for other SDK methods.

