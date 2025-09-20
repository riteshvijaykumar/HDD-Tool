use serde::Serialize;
use anyhow::Result;
use chrono::Utc;

#[derive(Debug, Serialize)]
pub struct Report {
    pub device: String,
    pub method: String,
    pub status: String,
    pub timestamp: String,
}

pub fn generate_report(device: &str) -> Result<Report> {
    Ok(Report {
        device: device.to_string(),
        method: "clear".to_string(),
        status: "success".to_string(),
        timestamp: Utc::now().to_rfc3339(),
    })
}
