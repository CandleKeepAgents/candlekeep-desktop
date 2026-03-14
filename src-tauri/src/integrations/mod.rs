pub mod amp;
pub mod claude_code;
pub mod codex;
pub mod cursor;

use serde::{Deserialize, Serialize};

use crate::platform::installer::ActionResult;
use crate::platform::PlatformInfo;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HostKind {
    ClaudeCode,
    Cursor,
    Codex,
    Amp,
}

#[allow(dead_code)]
impl HostKind {
    pub fn all() -> &'static [HostKind] {
        &[HostKind::ClaudeCode, HostKind::Cursor, HostKind::Codex, HostKind::Amp]
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            HostKind::ClaudeCode => "Claude Code",
            HostKind::Cursor => "Cursor",
            HostKind::Codex => "Codex",
            HostKind::Amp => "Amp",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            HostKind::ClaudeCode => "Access your CandleKeep library directly from Claude Code with AI-powered search and document management.",
            HostKind::Cursor => "Use your CandleKeep library as context in Cursor IDE for smarter code assistance.",
            HostKind::Codex => "Connect your CandleKeep library to OpenAI Codex for enhanced coding workflows.",
            HostKind::Amp => "Access your CandleKeep library from Amp for AI-powered development.",
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RequirementStatus {
    Satisfied,
    Missing,
    Unsupported,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize)]
pub struct Requirement {
    pub name: String,
    pub description: String,
    pub status: RequirementStatus,
}

#[derive(Debug, Clone, Serialize)]
pub struct IntegrationStatus {
    pub host: HostKind,
    pub host_installed: bool,
    pub integration_installed: bool,
    pub version: Option<String>,
    pub latest_version: Option<String>,
    pub install_method: String,
    pub status: RequirementStatus,
}

/// Trait that each host integration adapter implements.
#[allow(dead_code)]
pub trait HostIntegration: Send + Sync {
    fn kind(&self) -> HostKind;
    fn detect_host(&self, platform: &PlatformInfo) -> bool;
    fn detect_integration(&self) -> IntegrationStatus;
    fn install(&self) -> ActionResult;
    fn update(&self) -> ActionResult;
    fn repair(&self) -> ActionResult;
    fn requirements(&self, platform: &PlatformInfo) -> Vec<Requirement>;
}

/// Get the adapter for a given host kind.
pub fn get_adapter(kind: HostKind) -> Box<dyn HostIntegration> {
    match kind {
        HostKind::ClaudeCode => Box::new(claude_code::ClaudeCodeAdapter::new()),
        HostKind::Cursor => Box::new(cursor::CursorAdapter::new()),
        HostKind::Codex => Box::new(codex::CodexAdapter::new()),
        HostKind::Amp => Box::new(amp::AmpAdapter::new()),
    }
}

// --- Generic Tauri commands ---

#[tauri::command]
pub async fn list_integrations() -> Result<Vec<IntegrationStatus>, String> {
    let statuses: Vec<IntegrationStatus> = HostKind::all()
        .iter()
        .map(|kind| get_adapter(*kind).detect_integration())
        .collect();
    Ok(statuses)
}

#[tauri::command]
pub async fn check_integration(host: HostKind) -> Result<IntegrationStatus, String> {
    let adapter = get_adapter(host);
    Ok(adapter.detect_integration())
}

#[tauri::command]
pub async fn install_integration(host: HostKind) -> Result<ActionResult, String> {
    let adapter = get_adapter(host);
    Ok(adapter.install())
}

#[tauri::command]
pub async fn update_integration(host: HostKind) -> Result<ActionResult, String> {
    let adapter = get_adapter(host);
    Ok(adapter.update())
}

#[tauri::command]
pub async fn repair_integration(host: HostKind) -> Result<ActionResult, String> {
    let adapter = get_adapter(host);
    Ok(adapter.repair())
}
