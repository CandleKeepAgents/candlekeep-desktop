pub mod claude_code;
pub mod codex;
pub mod cursor;

use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct IntegrationInfo {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub installed: bool,
    pub version: Option<String>,
    pub latest_version: Option<String>,
    pub status: IntegrationStatus,
}

#[derive(Debug, Serialize, Clone)]
pub enum IntegrationStatus {
    Available,
    Installed,
    UpdateAvailable,
    ComingSoon,
}

pub trait Integration {
    fn name(&self) -> &str;
    fn display_name(&self) -> &str;
    fn description(&self) -> &str;
    fn is_installed(&self) -> bool;
    fn current_version(&self) -> Option<String>;
    fn latest_version(&self) -> Option<String>;
    fn status(&self) -> IntegrationStatus;
    fn info(&self) -> IntegrationInfo {
        IntegrationInfo {
            name: self.name().to_string(),
            display_name: self.display_name().to_string(),
            description: self.description().to_string(),
            installed: self.is_installed(),
            version: self.current_version(),
            latest_version: self.latest_version(),
            status: self.status(),
        }
    }
}
