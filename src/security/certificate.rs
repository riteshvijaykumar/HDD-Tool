use rsa::{RsaPrivateKey, RsaPublicKey, pkcs1v15::{SigningKey, VerifyingKey}};
use rsa::signature::{RandomizedSigner, Verifier, SignatureEncoding};
use sha2::{Sha256, Digest};
use base64::{Engine as _, engine::general_purpose};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::fs::{self};
use std::path::Path;

use crate::core::{WipeResult, WipeRequest, SecurityFeatures, WipeError, WipeErrorCode, WipeResult2};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErasureCertificate {
    pub certificate_id: Uuid,
    pub wipe_request_id: Uuid,
    pub issued_at: DateTime<Utc>,
    pub issuer: String,
    pub organization: String,
    pub drive_info: DriveInfo,
    pub wipe_details: WipeDetails,
    pub verification_details: VerificationDetails,
    pub compliance_standards: Vec<String>,
    pub signature: String,
    pub public_key: String,
    pub certificate_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveInfo {
    pub model: String,
    pub serial_number: String,
    pub firmware_version: String,
    pub total_capacity_gb: f64,
    pub native_capacity_gb: f64,
    pub had_hpa: bool,
    pub had_dco: bool,
    pub security_features: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WipeDetails {
    pub standard_used: String,
    pub passes_completed: u32,
    pub sectors_wiped: u64,
    pub start_time: DateTime<Utc>,
    pub completion_time: DateTime<Utc>,
    pub duration_minutes: u64,
    pub patterns_used: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationDetails {
    pub verification_performed: bool,
    pub sectors_verified: u64,
    pub verification_sample_rate: f64,
    pub pattern_verification_passed: bool,
    pub failed_sectors: u64,
    pub verification_time: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateAuthority {
    pub name: String,
    pub organization: String,
    pub private_key_pem: String,
    pub public_key_pem: String,
    pub certificate_counter: u64,
}

impl CertificateAuthority {
    pub fn new(name: String, organization: String) -> WipeResult2<Self> {
        let mut rng = rand::thread_rng();
        let bits = 2048;
        
        let private_key = RsaPrivateKey::new(&mut rng, bits)
            .map_err(|e| WipeError {
                code: WipeErrorCode::UnknownError,
                message: format!("Failed to generate private key: {}", e),
                sector: None,
            })?;

        let public_key = RsaPublicKey::from(&private_key);

        let private_key_pem = rsa::pkcs8::EncodePrivateKey::to_pkcs8_pem(&private_key, rsa::pkcs8::LineEnding::LF)
            .map_err(|e| WipeError {
                code: WipeErrorCode::UnknownError,
                message: format!("Failed to encode private key: {}", e),
                sector: None,
            })?;

        let public_key_pem = rsa::pkcs8::EncodePublicKey::to_public_key_pem(&public_key, rsa::pkcs8::LineEnding::LF)
            .map_err(|e| WipeError {
                code: WipeErrorCode::UnknownError,
                message: format!("Failed to encode public key: {}", e),
                sector: None,
            })?;

        Ok(Self {
            name,
            organization,
            private_key_pem: private_key_pem.to_string(),
            public_key_pem: public_key_pem.to_string(),
            certificate_counter: 0,
        })
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> WipeResult2<Self> {
        let content = fs::read_to_string(path)
            .map_err(|e| WipeError {
                code: WipeErrorCode::UnknownError,
                message: format!("Failed to read CA file: {}", e),
                sector: None,
            })?;

        serde_json::from_str(&content)
            .map_err(|e| WipeError {
                code: WipeErrorCode::UnknownError,
                message: format!("Failed to parse CA file: {}", e),
                sector: None,
            })
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> WipeResult2<()> {
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| WipeError {
                code: WipeErrorCode::UnknownError,
                message: format!("Failed to serialize CA: {}", e),
                sector: None,
            })?;

        fs::write(path, content)
            .map_err(|e| WipeError {
                code: WipeErrorCode::UnknownError,
                message: format!("Failed to write CA file: {}", e),
                sector: None,
            })
    }

    pub fn generate_certificate(
        &mut self,
        wipe_request: &WipeRequest,
        wipe_result: &WipeResult,
    ) -> WipeResult2<ErasureCertificate> {
        self.certificate_counter += 1;
        let certificate_id = Uuid::new_v4();

        // Extract drive information
        let drive_info = DriveInfo {
            model: wipe_result.drive_geometry.model.clone(),
            serial_number: wipe_result.drive_geometry.serial.clone(),
            firmware_version: wipe_result.drive_geometry.firmware.clone(),
            total_capacity_gb: wipe_result.drive_geometry.total_sectors as f64 * 512.0 / (1024.0 * 1024.0 * 1024.0),
            native_capacity_gb: wipe_result.drive_geometry.native_capacity as f64 / (1024.0 * 1024.0 * 1024.0),
            had_hpa: wipe_result.drive_geometry.has_hpa,
            had_dco: wipe_result.drive_geometry.has_dco,
            security_features: self.format_security_features(&wipe_result.security_features),
        };

        // Extract wipe details
        let wipe_details = WipeDetails {
            standard_used: format!("{:?}", wipe_request.standard),
            passes_completed: wipe_result.passes_completed,
            sectors_wiped: wipe_result.sectors_wiped,
            start_time: wipe_result.start_time,
            completion_time: wipe_result.completion_time,
            duration_minutes: wipe_result.duration_seconds / 60,
            patterns_used: self.get_pattern_descriptions(&wipe_request.standard),
        };

        // Extract verification details
        let verification_details = if let Some(ref validation) = wipe_result.validation_result {
            VerificationDetails {
                verification_performed: true,
                sectors_verified: validation.sectors_verified,
                verification_sample_rate: 0.1, // This should come from config
                pattern_verification_passed: validation.pattern_matches,
                failed_sectors: validation.failed_sectors.len() as u64,
                verification_time: Some(validation.completion_time),
            }
        } else {
            VerificationDetails {
                verification_performed: false,
                sectors_verified: 0,
                verification_sample_rate: 0.0,
                pattern_verification_passed: false,
                failed_sectors: 0,
                verification_time: None,
            }
        };

        // Create certificate (without signature initially)
        let mut certificate = ErasureCertificate {
            certificate_id,
            wipe_request_id: wipe_request.id,
            issued_at: Utc::now(),
            issuer: self.name.clone(),
            organization: self.organization.clone(),
            drive_info,
            wipe_details,
            verification_details,
            compliance_standards: vec![
                "NIST SP 800-88 Rev. 1".to_string(),
                "DoD 5220.22-M".to_string(),
            ],
            signature: String::new(),
            public_key: self.public_key_pem.clone(),
            certificate_hash: String::new(),
        };

        // Calculate hash of certificate content (excluding signature and hash fields)
        let certificate_content = self.get_certificate_content_for_signing(&certificate)?;
        let content_hash = self.calculate_hash(&certificate_content);
        certificate.certificate_hash = hex::encode(&content_hash);

        // Sign the certificate
        let signature = self.sign_data(&content_hash)?;
        certificate.signature = general_purpose::STANDARD.encode(&signature);

        Ok(certificate)
    }

    fn get_certificate_content_for_signing(&self, cert: &ErasureCertificate) -> WipeResult2<Vec<u8>> {
        // Create a version of the certificate without signature and hash for signing
        let signing_content = format!(
            "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
            cert.certificate_id,
            cert.wipe_request_id,
            cert.issued_at.to_rfc3339(),
            cert.issuer,
            cert.organization,
            cert.drive_info.serial_number,
            cert.drive_info.model,
            cert.wipe_details.standard_used,
            cert.wipe_details.passes_completed,
            cert.wipe_details.sectors_wiped,
            cert.wipe_details.start_time.to_rfc3339(),
            cert.wipe_details.completion_time.to_rfc3339(),
            cert.verification_details.verification_performed,
            cert.verification_details.sectors_verified,
            cert.verification_details.pattern_verification_passed,
            cert.verification_details.failed_sectors,
            cert.compliance_standards.join(","),
            cert.public_key
        );

        Ok(signing_content.into_bytes())
    }

    fn calculate_hash(&self, data: &[u8]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.finalize().into()
    }

    fn sign_data(&self, data: &[u8]) -> WipeResult2<Vec<u8>> {
        let private_key = rsa::pkcs8::DecodePrivateKey::from_pkcs8_pem(&self.private_key_pem)
            .map_err(|e| WipeError {
                code: WipeErrorCode::UnknownError,
                message: format!("Failed to decode private key: {}", e),
                sector: None,
            })?;

        let signing_key = SigningKey::<Sha256>::new_unprefixed(private_key);
        let mut rng = rand::thread_rng();
        
        let signature = signing_key.sign_with_rng(&mut rng, data);
        Ok(signature.to_bytes().to_vec())
    }

    pub fn verify_certificate(&self, certificate: &ErasureCertificate) -> WipeResult2<bool> {
        // Recreate the content that was signed
        let certificate_content = self.get_certificate_content_for_signing(certificate)?;
        let content_hash = self.calculate_hash(&certificate_content);

        // Verify the hash matches
        let calculated_hash = hex::encode(&content_hash);
        if calculated_hash != certificate.certificate_hash {
            return Ok(false);
        }

        // Verify the signature
        let signature_bytes = general_purpose::STANDARD.decode(&certificate.signature)
            .map_err(|e| WipeError {
                code: WipeErrorCode::UnknownError,
                message: format!("Failed to decode signature: {}", e),
                sector: None,
            })?;

        let public_key: RsaPublicKey = rsa::pkcs8::DecodePublicKey::from_public_key_pem(&certificate.public_key)
            .map_err(|e| WipeError {
                code: WipeErrorCode::UnknownError,
                message: format!("Failed to decode public key: {}", e),
                sector: None,
            })?;

        let verifying_key = rsa::pkcs1v15::VerifyingKey::<Sha256>::new_unprefixed(public_key);
        let signature = rsa::pkcs1v15::Signature::try_from(signature_bytes.as_slice())
            .map_err(|e| WipeError {
                code: WipeErrorCode::UnknownError,
                message: format!("Invalid signature format: {}", e),
                sector: None,
            })?;

        match verifying_key.verify(&content_hash, &signature) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    fn format_security_features(&self, features: &SecurityFeatures) -> Vec<String> {
        let mut result = Vec::new();
        
        if features.security_supported {
            result.push("ATA Security".to_string());
        }
        if features.enhanced_erase_supported {
            result.push("Enhanced Secure Erase".to_string());
        }
        if features.sanitize_supported {
            result.push("Sanitize Command".to_string());
        }
        if features.crypto_scramble_supported {
            result.push("Crypto Scramble".to_string());
        }
        
        if result.is_empty() {
            result.push("Basic".to_string());
        }
        
        result
    }

    fn get_pattern_descriptions(&self, standard: &crate::core::SanitizationStandard) -> Vec<String> {
        match standard {
            crate::core::SanitizationStandard::NIST80088Clear => {
                vec!["Single pass zeros".to_string()]
            }
            crate::core::SanitizationStandard::NIST80088Purge => {
                vec![
                    "Pass 1: Zeros".to_string(),
                    "Pass 2: Ones".to_string(),
                    "Pass 3: Random".to_string(),
                ]
            }
            crate::core::SanitizationStandard::DoD522022M => {
                vec![
                    "Pass 1: Zeros".to_string(),
                    "Pass 2: Ones".to_string(),
                    "Pass 3: Random".to_string(),
                    "Pass 4: Verification zeros".to_string(),
                ]
            }
        }
    }
}