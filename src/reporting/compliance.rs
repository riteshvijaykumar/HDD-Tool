use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReport {
    pub nist_compliance: bool,
    pub dod_compliance: bool,
    pub standards_met: Vec<String>,
    pub requirements_satisfied: Vec<String>,
}

impl ComplianceReport {
    pub fn new() -> Self {
        Self {
            nist_compliance: false,
            dod_compliance: false,
            standards_met: Vec::new(),
            requirements_satisfied: Vec::new(),
        }
    }
}