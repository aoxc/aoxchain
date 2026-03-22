use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractMetadata {
    pub display_name: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub organization: Option<String>,
    pub source_reference: Option<String>,
    pub tags: Vec<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub audit_reference: Option<String>,
    pub notes: Option<String>,
}
