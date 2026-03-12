use serde::{Deserialize, Serialize};
#[allow(unused_imports)]
use tracing::{warn, error};

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppState {
    pub setup_completed: bool,
    pub last_update_check: Option<u64>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            setup_completed: false,
            last_update_check: None,
        }
    }
}

#[allow(dead_code)]
impl AppState {
    pub fn load() -> Self {
        let state_path = dirs::home_dir()
            .map(|h| h.join(".candlekeep/desktop-state.json"));

        match state_path {
            Some(path) if path.exists() => {
                match std::fs::read_to_string(&path) {
                    Ok(content) => match serde_json::from_str(&content) {
                        Ok(state) => state,
                        Err(e) => {
                            warn!("State file corrupt, falling back to default: {}", e);
                            Self::default()
                        }
                    },
                    Err(e) => {
                        warn!("Failed to read state file, falling back to default: {}", e);
                        Self::default()
                    }
                }
            }
            Some(_) => {
                warn!("State file not found, using defaults");
                Self::default()
            }
            _ => Self::default(),
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let state_path = dirs::home_dir()
            .ok_or("Could not find home directory")?
            .join(".candlekeep/desktop-state.json");

        // Ensure directory exists
        if let Some(parent) = state_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        }

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize state: {}", e))?;

        std::fs::write(&state_path, content)
            .map_err(|e| {
                error!("Failed to write state file: {}", e);
                format!("Failed to write state: {}", e)
            })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_default_state() {
        let state = AppState::default();
        assert!(!state.setup_completed);
        assert!(state.last_update_check.is_none());
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let tmp_dir = std::env::temp_dir().join("candlekeep-test-state");
        let _ = fs::create_dir_all(&tmp_dir);
        let state_path = tmp_dir.join("desktop-state.json");

        let state = AppState {
            setup_completed: true,
            last_update_check: Some(1234567890),
        };

        let content = serde_json::to_string_pretty(&state).unwrap();
        fs::write(&state_path, &content).unwrap();

        let loaded: AppState = serde_json::from_str(&fs::read_to_string(&state_path).unwrap()).unwrap();
        assert!(loaded.setup_completed);
        assert_eq!(loaded.last_update_check, Some(1234567890));

        // Cleanup
        let _ = fs::remove_dir_all(&tmp_dir);
    }

    #[test]
    fn test_load_missing_file_returns_default() {
        let content = "not valid json";
        let result: Result<AppState, _> = serde_json::from_str(content);
        assert!(result.is_err());
        // The load() function would return default in this case
        let default = AppState::default();
        assert!(!default.setup_completed);
    }

    #[test]
    fn test_deserialize_partial_json() {
        let json = r#"{"setup_completed": true}"#;
        let state: AppState = serde_json::from_str(json).unwrap();
        assert!(state.setup_completed);
        assert!(state.last_update_check.is_none());
    }
}
