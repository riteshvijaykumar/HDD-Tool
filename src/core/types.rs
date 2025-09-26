use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SanitizationStandard {
    NIST_SP_800_88_R1,  // NIST SP 800-88 Rev. 1
    DoD_5220_22_M,      // DoD 5220.22-M
    AFSSI_5020,         // AFSSI-5020
    BSI_2011_VS,        // BSI 2011-VS
    NAVSO_P_5239_26,    // NAVSO P-5239-26
}

impl SanitizationStandard {
    pub fn get_pass_count(&self) -> u32 {
        match self {
            SanitizationStandard::NIST_SP_800_88_R1 => 1,
            SanitizationStandard::DoD_5220_22_M => 3,
            SanitizationStandard::AFSSI_5020 => 3,
            SanitizationStandard::BSI_2011_VS => 2,
            SanitizationStandard::NAVSO_P_5239_26 => 3,
        }
    }

    pub fn get_patterns(&self) -> Vec<u8> {
        match self {
            SanitizationStandard::NIST_SP_800_88_R1 => vec![0x00],
            SanitizationStandard::DoD_5220_22_M => vec![0x00, 0xFF, 0x96],
            SanitizationStandard::AFSSI_5020 => vec![0x00, 0xFF, 0x96],
            SanitizationStandard::BSI_2011_VS => vec![0x00, 0xFF],
            SanitizationStandard::NAVSO_P_5239_26 => vec![0x01, 0x27, 0x96],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TargetType {
    HDD,
    SSD,
    Flash,
    Optical,
    Tape,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WipeConfiguration {
    pub standard: SanitizationStandard,
    pub target_type: TargetType,
    pub verify_writes: bool,
    pub generate_report: bool,
    pub certificate_required: bool,
    pub buffer_size: usize,
    pub verification_sample_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WipeRequest {
    pub id: Uuid,
    pub target_path: String,
    pub target_type: TargetType,
    pub standard: SanitizationStandard,
    pub passes: u32,
    pub verify_erasure: bool,
    pub generate_certificate: bool,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveGeometry {
    pub model: String,
    pub serial: String,
    pub firmware: String,
    pub total_sectors: u64,
    pub sector_size: u64,
    pub user_capacity: u64,
    pub native_capacity: u64,
    pub has_hpa: bool,
    pub has_dco: bool,
    pub hpa_size: u64,
    pub dco_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityFeatures {
    pub security_supported: bool,
    pub security_enabled: bool,
    pub security_locked: bool,
    pub security_frozen: bool,
    pub enhanced_erase_supported: bool,
    pub sanitize_supported: bool,
    pub crypto_scramble_supported: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WipeProgress {
    pub request_id: Uuid,
    pub current_pass: u32,
    pub total_passes: u32,
    pub sectors_processed: u64,
    pub total_sectors: u64,
    pub percentage: f64,
    pub current_operation: String,
    pub estimated_completion: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub sectors_verified: u64,
    pub failed_sectors: Vec<u64>,
    pub pattern_matches: bool,
    pub checksum_valid: bool,
    pub completion_time: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WipeResult {
    pub request_id: Uuid,
    pub success: bool,
    pub start_time: DateTime<Utc>,
    pub completion_time: DateTime<Utc>,
    pub duration_seconds: u64,
    pub sectors_wiped: u64,
    pub passes_completed: u32,
    pub validation_result: Option<ValidationResult>,
    pub error_message: Option<String>,
    pub drive_geometry: DriveGeometry,
    pub security_features: SecurityFeatures,
}

#[derive(Debug, Clone)]
pub struct WipeError {
    pub code: WipeErrorCode,
    pub message: String,
    pub sector: Option<u64>,
}

#[derive(Debug, Clone)]
pub enum WipeErrorCode {
    AccessDenied,
    DriveNotFound,
    HardwareError,
    VerificationFailed,
    HPAUnlockFailed,
    DCOUnlockFailed,
    SecurityLocked,
    InvalidPattern,
    InsufficientPrivileges,
    UnknownError,
}

impl std::fmt::Display for WipeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.code, self.message)
    }
}

impl std::error::Error for WipeError {}

pub type WipeResult2<T> = Result<T, WipeError>;