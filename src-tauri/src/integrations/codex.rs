use super::{Integration, IntegrationStatus};

pub struct CodexIntegration;

impl CodexIntegration {
    pub fn new() -> Self {
        Self
    }
}

impl Integration for CodexIntegration {
    fn name(&self) -> &str {
        "codex"
    }

    fn display_name(&self) -> &str {
        "Codex"
    }

    fn description(&self) -> &str {
        "CandleKeep integration for OpenAI Codex — coming soon"
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
