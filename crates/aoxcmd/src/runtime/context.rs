use crate::config::settings::Settings;
use crate::keys::material::KeyMaterialSummary;

#[derive(Debug, Clone)]
pub struct RuntimeContext {
    pub settings: Settings,
    pub key_summary: KeyMaterialSummary,
}
