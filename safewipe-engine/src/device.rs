use anyhow::Result;

#[derive(Debug, Clone)]
pub struct Device {
    pub path: String,
    pub size: u64,
    pub device_type: String,
}

/// Placeholder device detection
pub fn list_devices() -> Result<Vec<Device>> {
    // Later: implement per-OS detection
    Ok(vec![
        Device {
            path: "/dev/sda".to_string(),
            size: 512_000_000_000,
            device_type: "SSD".to_string(),
        },
        Device {
            path: "/dev/sdb".to_string(),
            size: 1_000_000_000_000,
            device_type: "HDD".to_string(),
        },
    ])
}
