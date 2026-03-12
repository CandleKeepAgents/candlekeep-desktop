use super::{Integration, IntegrationStatus};

pub struct CursorIntegration;

impl CursorIntegration {
    pub fn new() -> Self {
        Self
    }
}

impl Integration for CursorIntegration {
    fn name(&self) -> &str {
        "cursor"
    }

    fn display_name(&self) -> &str {
        "Cursor"
    }

    fn description(&self) -> &str {
        "CandleKeep integration for Cursor IDE — coming soon"
    }

    fn is_installed(&self) -> bool {
        false
    }

    fn current_version(&self) -> Option<String> {
        None
    }

    fn latest_version(&self) -> Option<String> {
        None
    }

    fn status(&self) -> IntegrationStatus {
        IntegrationStatus::ComingSoon
    }
}
