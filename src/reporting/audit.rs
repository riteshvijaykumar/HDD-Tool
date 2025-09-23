use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditTrail {
    pub event_id: String,
    pub timestamp: DateTime<Utc>,
    pub event_type: String,
    pub description: String,
    pub metadata: HashMap<String, String>,
}

impl AuditTrail {
    pub fn new(event_type: &str, description: &str) -> Self {
        Self {
            event_id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            event_type: event_type.to_string(),
            description: description.to_string(),
            metadata: HashMap::new(),
        }
    }
}