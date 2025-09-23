use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WipeSystemConfiguration {
    pub buffer_size: usize,
    pub max_threads: usize,
    pub verification_sample_rate: f64, // 0.0 to 1.0
    pub enable_progress_reporting: bool,
    pub certificate_generation: bool,
    pub audit_logging: bool,
}

impl Default for WipeSystemConfiguration {
    fn default() -> Self {
        Self {
            buffer_size: 16 * 1024 * 1024, // 16MB
            max_threads: num_cpus::get().min(8),
            verification_sample_rate: 0.1, // 10% sampling
            enable_progress_reporting: true,
            certificate_generation: true,
            audit_logging: true,
        }
    }
}

pub const NIST_CLEAR_PATTERNS: &[u8] = &[0x00];
pub const NIST_PURGE_PATTERNS: &[&[u8]] = &[
    &[0x00], // Pass 1: Zeros
    &[0xFF], // Pass 2: Ones  
    // Pass 3: Random (generated dynamically)
];

pub const DOD_522022M_PATTERNS: &[&[u8]] = &[
    &[0x00], // Pass 1: Zeros
    &[0xFF], // Pass 2: Ones
    &[0x96], // Pass 3: DoD pattern
    &[0x96], // Pass 4: DoD pattern verification
];

pub const VERIFICATION_BLOCK_SIZE: usize = 64 * 1024; // 64KB blocks
pub const MAX_RETRY_ATTEMPTS: u32 = 3;
pub const PROGRESS_UPDATE_INTERVAL: u64 = 100; // milliseconds