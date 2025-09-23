use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use chrono::Utc;
use uuid::Uuid;

use crate::core::{
    SanitizationStandard, TargetType, WipeRequest, WipeProgress, WipeResult, 
    WipeConfiguration, WipeError, WipeErrorCode, WipeResult2
};
use crate::hardware::SecureSanitizer;
use crate::security::{CertificateAuthority, ReportGenerator};

pub struct WipeEngine {
    config: WipeConfiguration,
    certificate_authority: Arc<Mutex<CertificateAuthority>>,
    report_generator: ReportGenerator,
}

impl WipeEngine {
    pub fn new(config: WipeConfiguration) -> WipeResult2<Self> {
        let ca = CertificateAuthority::new(
            "SecureWipe Certificate Authority".to_string(),
            "Data Security Solutions Inc.".to_string(),
        )?;

        // Save the CA to file for persistence
        ca.save_to_file("certificates/ca.json")?;

        Ok(Self {
            config,
            certificate_authority: Arc::new(Mutex::new(ca)),
            report_generator: ReportGenerator::new(),
        })
    }

    pub fn execute_secure_wipe(
        &self,
        target_path: String,
        target_type: TargetType,
        standard: SanitizationStandard,
        verify_erasure: bool,
        generate_certificate: bool,
    ) -> WipeResult2<(WipeResult, Option<String>)> {
        // Create wipe request
        let request = WipeRequest {
            id: Uuid::new_v4(),
            target_path,
            target_type,
            standard,
            passes: match standard {
                SanitizationStandard::NIST_SP_800_88_R1 => 1,
                SanitizationStandard::DoD_5220_22_M => 3,
                SanitizationStandard::AFSSI_5020 => 3,
                SanitizationStandard::BSI_2011_VS => 2,
                SanitizationStandard::NAVSO_P_5239_26 => 3,
            },
            verify_erasure,
            generate_certificate,
            timestamp: Utc::now(),
        };

        println!("Starting secure wipe operation");
        println!("Request ID: {}", request.id);
        println!("Target: {}", request.target_path);
        println!("Standard: {:?}", request.standard);

        // Create progress channel
        let (progress_tx, progress_rx) = mpsc::channel::<WipeProgress>();
        
        // Create sanitizer with progress reporting
        let sanitizer = SecureSanitizer::new(self.config.clone())
            .with_progress_reporting(progress_tx);

        // Start progress monitoring in a separate thread
        let request_id = request.id;
        thread::spawn(move || {
            while let Ok(progress) = progress_rx.recv() {
                println!(
                    "Progress: {:.1}% - Pass {}/{} - {} - {} sectors processed",
                    progress.percentage,
                    progress.current_pass,
                    progress.total_passes,
                    progress.current_operation,
                    progress.sectors_processed
                );
            }
        });

        // Execute the wipe
        let wipe_result = sanitizer.execute_wipe(request.clone())?;

        // Generate certificate if requested
        let certificate_path = if generate_certificate && wipe_result.success {
            let mut ca = self.certificate_authority.lock().unwrap();
            let certificate = ca.generate_certificate(&request, &wipe_result)?;
            
            // Generate timestamp for unique filenames
            let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
            let cert_filename = format!("reports/certificate_{}_{}.pdf", 
                                      wipe_result.drive_geometry.serial, timestamp);
            let json_filename = format!("reports/audit_{}_{}.json", 
                                       wipe_result.drive_geometry.serial, timestamp);

            // Generate PDF certificate
            self.report_generator.generate_pdf_report(&certificate, &cert_filename)?;
            
            // Generate JSON audit report
            self.report_generator.generate_json_report(
                &certificate, 
                &request, 
                &wipe_result, 
                &json_filename
            )?;

            // Save updated CA (incremented counter)
            ca.save_to_file("certificates/ca.json")?;

            println!("Certificate generated: {}", cert_filename);
            println!("Audit report generated: {}", json_filename);

            Some(cert_filename)
        } else {
            None
        };

        Ok((wipe_result, certificate_path))
    }

    pub fn validate_certificate(&self, certificate_path: &str) -> WipeResult2<bool> {
        // Load certificate from JSON file
        let content = std::fs::read_to_string(certificate_path)
            .map_err(|e| WipeError {
                code: WipeErrorCode::UnknownError,
                message: format!("Failed to read certificate: {}", e),
                sector: None,
            })?;

        let certificate: crate::security::ErasureCertificate = serde_json::from_str(&content)
            .map_err(|e| WipeError {
                code: WipeErrorCode::UnknownError,
                message: format!("Failed to parse certificate: {}", e),
                sector: None,
            })?;

        // Verify with CA
        let ca = self.certificate_authority.lock().unwrap();
        ca.verify_certificate(&certificate)
    }

    pub fn get_drive_info(&self, drive_path: &str) -> WipeResult2<(crate::core::DriveGeometry, crate::core::SecurityFeatures)> {
        use crate::hardware::DriveInterface;
        
        let drive = DriveInterface::new(drive_path)?;
        let identify_data = drive.identify_device()?;
        let geometry = drive.parse_drive_geometry(&identify_data);
        let security = drive.parse_security_features(&identify_data);
        
        Ok((geometry, security))
    }
}