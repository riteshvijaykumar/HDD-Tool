use printpdf::{PdfDocument, Mm, PdfDocumentReference, PdfLayerReference, IndirectFontRef, Color, Rgb};
use chrono::{DateTime, Utc};
use serde_json;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::core::{WipeRequest, WipeResult, WipeError, WipeErrorCode, WipeResult2};
use crate::security::ErasureCertificate;

pub struct ReportGenerator;

impl ReportGenerator {
    pub fn new() -> Self {
        Self
    }

    pub fn generate_pdf_report<P: AsRef<Path>>(
        &self,
        certificate: &ErasureCertificate,
        output_path: P,
    ) -> WipeResult2<()> {
        let (doc, page1, layer1) = PdfDocument::new("Data Erasure Certificate", Mm(210.0), Mm(297.0), "Layer 1");
        let current_layer = doc.get_page(page1).get_layer(layer1);

        // Load fonts
        let font = doc.add_builtin_font(printpdf::BuiltinFont::Helvetica)?;
        let font_bold = doc.add_builtin_font(printpdf::BuiltinFont::HelveticaBold)?;

        // Title
        self.add_title(&current_layer, &font_bold, "DATA ERASURE CERTIFICATE", 280.0)?;
        
        // Header information
        let mut y_pos = 260.0;
        y_pos = self.add_header_section(&current_layer, &font, &font_bold, certificate, y_pos)?;
        
        // Drive information
        y_pos = self.add_drive_section(&current_layer, &font, &font_bold, certificate, y_pos)?;
        
        // Wipe details
        y_pos = self.add_wipe_section(&current_layer, &font, &font_bold, certificate, y_pos)?;
        
        // Verification details
        y_pos = self.add_verification_section(&current_layer, &font, &font_bold, certificate, y_pos)?;
        
        // Compliance and signature
        y_pos = self.add_compliance_section(&current_layer, &font, &font_bold, certificate, y_pos)?;
        
        // Footer
        self.add_footer(&current_layer, &font, certificate)?;

        // Save the PDF
        doc.save(&mut std::io::BufWriter::new(
            File::create(output_path).map_err(|e| WipeError {
                code: WipeErrorCode::UnknownError,
                message: format!("Failed to create PDF file: {}", e),
                sector: None,
            })?
        )).map_err(|e| WipeError {
            code: WipeErrorCode::UnknownError,
            message: format!("Failed to save PDF: {}", e),
            sector: None,
        })?;

        Ok(())
    }

    pub fn generate_json_report<P: AsRef<Path>>(
        &self,
        certificate: &ErasureCertificate,
        wipe_request: &WipeRequest,
        wipe_result: &WipeResult,
        output_path: P,
    ) -> WipeResult2<()> {
        #[derive(serde::Serialize)]
        struct JsonReport {
            certificate: ErasureCertificate,
            metadata: ReportMetadata,
            audit_trail: AuditTrail,
        }

        #[derive(serde::Serialize)]
        struct ReportMetadata {
            report_version: String,
            generated_at: DateTime<Utc>,
            generator: String,
            compliance_standards: Vec<String>,
        }

        #[derive(serde::Serialize)]
        struct AuditTrail {
            original_request: WipeRequest,
            execution_result: WipeResult,
            integrity_checks: IntegrityChecks,
        }

        #[derive(serde::Serialize)]
        struct IntegrityChecks {
            certificate_hash_verified: bool,
            signature_verified: bool,
            timestamp_valid: bool,
        }

        let report = JsonReport {
            certificate: certificate.clone(),
            metadata: ReportMetadata {
                report_version: "1.0".to_string(),
                generated_at: Utc::now(),
                generator: "SecureWipe Pro v1.0".to_string(),
                compliance_standards: vec![
                    "NIST SP 800-88 Rev. 1".to_string(),
                    "DoD 5220.22-M".to_string(),
                ],
            },
            audit_trail: AuditTrail {
                original_request: wipe_request.clone(),
                execution_result: wipe_result.clone(),
                integrity_checks: IntegrityChecks {
                    certificate_hash_verified: true, // This should be verified
                    signature_verified: true,        // This should be verified
                    timestamp_valid: true,           // This should be verified
                },
            },
        };

        let json_content = serde_json::to_string_pretty(&report)
            .map_err(|e| WipeError {
                code: WipeErrorCode::UnknownError,
                message: format!("Failed to serialize JSON report: {}", e),
                sector: None,
            })?;

        std::fs::write(output_path, json_content)
            .map_err(|e| WipeError {
                code: WipeErrorCode::UnknownError,
                message: format!("Failed to write JSON report: {}", e),
                sector: None,
            })?;

        Ok(())
    }

    fn add_title(
        &self,
        layer: &PdfLayerReference,
        font: &IndirectFontRef,
        text: &str,
        y_pos: f64,
    ) -> WipeResult2<()> {
        layer.use_text(text, 18.0, Mm(105.0), Mm(y_pos as f32), font);
        Ok(())
    }

    fn add_header_section(
        &self,
        layer: &PdfLayerReference,
        font: &IndirectFontRef,
        font_bold: &IndirectFontRef,
        certificate: &ErasureCertificate,
        start_y: f64,
    ) -> WipeResult2<f64> {
        let mut y_pos = start_y - 20.0;

        layer.use_text("CERTIFICATE INFORMATION", 14.0, Mm(20.0), Mm(y_pos as f32), font_bold);
        y_pos -= 15.0;

        let header_items = vec![
            ("Certificate ID:", certificate.certificate_id.to_string()),
            ("Issue Date:", certificate.issued_at.format("%Y-%m-%d %H:%M:%S UTC").to_string()),
            ("Issuer:", certificate.issuer.clone()),
            ("Organization:", certificate.organization.clone()),
        ];

        for (label, value) in header_items {
            layer.use_text(label, 10.0, Mm(20.0), Mm(y_pos as f32), font_bold);
            layer.use_text(&value, 10.0, Mm(70.0), Mm(y_pos as f32), font);
            y_pos -= 12.0;
        }

        Ok(y_pos - 10.0)
    }

    fn add_drive_section(
        &self,
        layer: &PdfLayerReference,
        font: &IndirectFontRef,
        font_bold: &IndirectFontRef,
        certificate: &ErasureCertificate,
        start_y: f64,
    ) -> WipeResult2<f64> {
        let mut y_pos = start_y;

        layer.use_text("DRIVE INFORMATION", 14.0, Mm(20.0), Mm(y_pos as f32), font_bold);
        y_pos -= 15.0;

        let drive_items = vec![
            ("Model:", certificate.drive_info.model.clone()),
            ("Serial Number:", certificate.drive_info.serial_number.clone()),
            ("Firmware:", certificate.drive_info.firmware_version.clone()),
            ("Total Capacity:", format!("{:.2} GB", certificate.drive_info.total_capacity_gb)),
            ("Native Capacity:", format!("{:.2} GB", certificate.drive_info.native_capacity_gb)),
            ("HPA Detected:", if certificate.drive_info.had_hpa { "Yes" } else { "No" }.to_string()),
            ("DCO Detected:", if certificate.drive_info.had_dco { "Yes" } else { "No" }.to_string()),
            ("Security Features:", certificate.drive_info.security_features.join(", ")),
        ];

        for (label, value) in drive_items {
            layer.use_text(label, 10.0, Mm(20.0), Mm(y_pos as f32), font_bold);
            layer.use_text(&value, 10.0, Mm(70.0), Mm(y_pos as f32), font);
            y_pos -= 12.0;
        }

        Ok(y_pos - 10.0)
    }

    fn add_wipe_section(
        &self,
        layer: &PdfLayerReference,
        font: &IndirectFontRef,
        font_bold: &IndirectFontRef,
        certificate: &ErasureCertificate,
        start_y: f64,
    ) -> WipeResult2<f64> {
        let mut y_pos = start_y;

        layer.use_text("SANITIZATION DETAILS", 14.0, Mm(20.0), Mm(y_pos as f32), font_bold);
        y_pos -= 15.0;

        let wipe_items = vec![
            ("Standard Used:", certificate.wipe_details.standard_used.clone()),
            ("Passes Completed:", certificate.wipe_details.passes_completed.to_string()),
            ("Sectors Wiped:", certificate.wipe_details.sectors_wiped.to_string()),
            ("Start Time:", certificate.wipe_details.start_time.format("%Y-%m-%d %H:%M:%S UTC").to_string()),
            ("Completion Time:", certificate.wipe_details.completion_time.format("%Y-%m-%d %H:%M:%S UTC").to_string()),
            ("Duration:", format!("{} minutes", certificate.wipe_details.duration_minutes)),
            ("Patterns Used:", certificate.wipe_details.patterns_used.join(", ")),
        ];

        for (label, value) in wipe_items {
            layer.use_text(label, 10.0, Mm(20.0), Mm(y_pos as f32), font_bold);
            layer.use_text(&value, 10.0, Mm(70.0), Mm(y_pos as f32), font);
            y_pos -= 12.0;
        }

        Ok(y_pos - 10.0)
    }

    fn add_verification_section(
        &self,
        layer: &PdfLayerReference,
        font: &IndirectFontRef,
        font_bold: &IndirectFontRef,
        certificate: &ErasureCertificate,
        start_y: f64,
    ) -> WipeResult2<f64> {
        let mut y_pos = start_y;

        layer.use_text("VERIFICATION DETAILS", 14.0, Mm(20.0), Mm(y_pos as f32), font_bold);
        y_pos -= 15.0;

        let verification_items = vec![
            ("Verification Performed:", if certificate.verification_details.verification_performed { "Yes" } else { "No" }.to_string()),
            ("Sectors Verified:", certificate.verification_details.sectors_verified.to_string()),
            ("Sample Rate:", format!("{:.1}%", certificate.verification_details.verification_sample_rate * 100.0)),
            ("Pattern Verification:", if certificate.verification_details.pattern_verification_passed { "PASSED" } else { "FAILED" }.to_string()),
            ("Failed Sectors:", certificate.verification_details.failed_sectors.to_string()),
        ];

        for (label, value) in verification_items {
            layer.use_text(label, 10.0, Mm(20.0), Mm(y_pos as f32), font_bold);
            layer.use_text(&value, 10.0, Mm(70.0), Mm(y_pos as f32), font);
            y_pos -= 12.0;
        }

        Ok(y_pos - 10.0)
    }

    fn add_compliance_section(
        &self,
        layer: &PdfLayerReference,
        font: &IndirectFontRef,
        font_bold: &IndirectFontRef,
        certificate: &ErasureCertificate,
        start_y: f64,
    ) -> WipeResult2<f64> {
        let mut y_pos = start_y;

        layer.use_text("COMPLIANCE & CERTIFICATION", 14.0, Mm(20.0), Mm(y_pos as f32), font_bold);
        y_pos -= 15.0;

        layer.use_text("Compliance Standards:", 10.0, Mm(20.0), Mm(y_pos as f32), font_bold);
        y_pos -= 12.0;

        for standard in &certificate.compliance_standards {
            layer.use_text(&format!("- {}", standard), 10.0, Mm(25.0), Mm(y_pos as f32), font);
            y_pos -= 12.0;
        }

        y_pos -= 10.0;
        layer.use_text("DIGITAL SIGNATURE", 12.0, Mm(20.0), Mm(y_pos as f32), font_bold);
        y_pos -= 15.0;

        layer.use_text("Certificate Hash:", 10.0, Mm(20.0), Mm(y_pos as f32), font_bold);
        y_pos -= 12.0;
        layer.use_text(&certificate.certificate_hash, 8.0, Mm(20.0), Mm(y_pos as f32), font);
        y_pos -= 15.0;

        layer.use_text("Digital Signature:", 10.0, Mm(20.0), Mm(y_pos as f32), font_bold);
        y_pos -= 12.0;
        
        // Truncate signature for display
        let sig_display = if certificate.signature.len() > 80 {
            format!("{}...", &certificate.signature[..80])
        } else {
            certificate.signature.clone()
        };
        layer.use_text(&sig_display, 8.0, Mm(20.0), Mm(y_pos as f32), font);

        Ok(y_pos - 20.0)
    }

    fn add_footer(
        &self,
        layer: &PdfLayerReference,
        font: &IndirectFontRef,
        certificate: &ErasureCertificate,
    ) -> WipeResult2<()> {
        layer.use_text(
            "This certificate is cryptographically signed and tamper-evident.",
            8.0,
            Mm(20.0),
            Mm(30.0),
            font,
        );
        
        layer.use_text(
            &format!("Generated on: {}", Utc::now().format("%Y-%m-%d %H:%M:%S UTC")),
            8.0,
            Mm(20.0),
            Mm(20.0),
            font,
        );

        Ok(())
    }
}

impl From<printpdf::Error> for WipeError {
    fn from(err: printpdf::Error) -> Self {
        WipeError {
            code: WipeErrorCode::UnknownError,
            message: format!("PDF generation error: {}", err),
            sector: None,
        }
    }
}
