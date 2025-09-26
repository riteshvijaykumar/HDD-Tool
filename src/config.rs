use serde::{Deserialize, Serialize};
use std::fs;
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server_url: String,
    pub enable_server_sync: bool,
    pub auto_upload_certificates: bool,
    pub local_storage_only: bool,
    pub connection_timeout_seconds: u64,
    pub retry_attempts: u32,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server_url: "http://localhost:3000".to_string(),
            enable_server_sync: false,
            auto_upload_certificates: true,
            local_storage_only: true,
            connection_timeout_seconds: 30,
            retry_attempts: 3,
        }
    }
}

impl AppConfig {
    pub fn load() -> Self {
        // First, try to load from environment variable
        if let Ok(server_url) = env::var("HDD_TOOL_SERVER_URL") {
            let mut config = Self::default();
            config.server_url = server_url;
            config.enable_server_sync = true;
            config.local_storage_only = false;
            return config;
        }
        
        // Then try to load from config file
        if let Ok(config_str) = fs::read_to_string("config.json") {
            if let Ok(mut config) = serde_json::from_str::<AppConfig>(&config_str) {
                // Override with environment variables if present
                if let Ok(url) = env::var("HDD_TOOL_SERVER_URL") {
                    config.server_url = url;
                    config.enable_server_sync = true;
                    config.local_storage_only = false;
                }
                return config;
            }
        }
        
        // Fall back to default (local only)
        Self::default()
    }
    
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_str = serde_json::to_string_pretty(self)?;
        fs::write("config.json", config_str)?;
        Ok(())
    }
    
    pub fn is_server_enabled(&self) -> bool {
        self.enable_server_sync && !self.local_storage_only
    }
    
    pub fn get_api_url(&self, endpoint: &str) -> String {
        format!("{}/api/{}", self.server_url.trim_end_matches('/'), endpoint.trim_start_matches('/'))
    }
    
    pub fn get_dashboard_url(&self) -> String {
        format!("{}/dashboard", self.server_url.trim_end_matches('/'))
    }
}