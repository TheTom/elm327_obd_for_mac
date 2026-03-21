use serde::Deserialize;
use std::path::Path;

use crate::error::{BridgeError, Result};

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default = "default_device")]
    pub device: String,

    #[serde(default = "default_baud")]
    pub baud_rate: u32,

    #[serde(default = "default_com")]
    pub wine_com_port: String,

    #[serde(default)]
    pub auto_reconnect: bool,

    #[serde(default = "default_true")]
    pub logging: bool,

    #[serde(default)]
    pub log_level: Option<String>,

    #[serde(default)]
    pub wine_prefix: Option<String>,
}

fn default_device() -> String {
    "auto".to_string()
}

fn default_baud() -> u32 {
    38400
}

fn default_com() -> String {
    "COM3".to_string()
}

fn default_true() -> bool {
    true
}

impl Default for Config {
    fn default() -> Self {
        Self {
            device: default_device(),
            baud_rate: default_baud(),
            wine_com_port: default_com(),
            auto_reconnect: false,
            logging: true,
            log_level: None,
            wine_prefix: None,
        }
    }
}

impl Config {
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            BridgeError::Config(format!("Failed to read {}: {}", path.display(), e))
        })?;
        serde_yaml::from_str(&content).map_err(|e| {
            BridgeError::Config(format!("Failed to parse {}: {}", path.display(), e))
        })
    }

    /// Returns the Wine prefix path.
    /// Uses the configured `wine_prefix` if set, otherwise defaults to `$HOME/.wine`.
    /// Falls back to `/tmp/.wine` if `$HOME` is not set.
    pub fn wine_prefix_path(&self) -> std::path::PathBuf {
        if let Some(ref prefix) = self.wine_prefix {
            std::path::PathBuf::from(prefix)
        } else {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
            std::path::PathBuf::from(home).join(".wine")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn load_valid_yaml() {
        let yaml = r#"
device: /dev/ttyUSB1
baud_rate: 115200
wine_com_port: COM5
auto_reconnect: true
logging: false
log_level: debug
wine_prefix: /opt/wine
"#;
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(yaml.as_bytes()).unwrap();

        let cfg = Config::load(tmp.path()).unwrap();
        assert_eq!(cfg.device, "/dev/ttyUSB1");
        assert_eq!(cfg.baud_rate, 115200);
        assert_eq!(cfg.wine_com_port, "COM5");
        assert!(cfg.auto_reconnect);
        assert!(!cfg.logging);
        assert_eq!(cfg.log_level.as_deref(), Some("debug"));
        assert_eq!(cfg.wine_prefix.as_deref(), Some("/opt/wine"));
    }

    #[test]
    fn defaults_are_correct() {
        let cfg = Config::default();
        assert_eq!(cfg.device, "auto");
        assert_eq!(cfg.baud_rate, 38400);
        assert_eq!(cfg.wine_com_port, "COM3");
        assert!(!cfg.auto_reconnect);
        assert!(cfg.logging);
        assert!(cfg.log_level.is_none());
        assert!(cfg.wine_prefix.is_none());
    }

    #[test]
    fn missing_fields_get_defaults() {
        // Empty YAML doc — all fields should fall back to serde defaults
        let yaml = "---\n";
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(yaml.as_bytes()).unwrap();

        let cfg = Config::load(tmp.path()).unwrap();
        assert_eq!(cfg.device, "auto");
        assert_eq!(cfg.baud_rate, 38400);
        assert_eq!(cfg.wine_com_port, "COM3");
        assert!(!cfg.auto_reconnect);
        assert!(cfg.logging);
    }

    #[test]
    fn invalid_yaml_returns_error() {
        let yaml = "{{{{not valid yaml at all";
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(yaml.as_bytes()).unwrap();

        let err = Config::load(tmp.path()).unwrap_err();
        assert!(matches!(err, BridgeError::Config(_)));
        assert!(err.to_string().contains("Failed to parse"));
    }

    #[test]
    fn load_nonexistent_file_returns_error() {
        let err = Config::load(Path::new("/nonexistent/config.yaml")).unwrap_err();
        assert!(matches!(err, BridgeError::Config(_)));
        assert!(err.to_string().contains("Failed to read"));
    }

    #[test]
    fn wine_prefix_path_default() {
        let cfg = Config::default();
        let path = cfg.wine_prefix_path();
        // Should end with .wine regardless of $HOME value
        assert!(path.ends_with(".wine"));
    }

    #[test]
    fn wine_prefix_path_custom() {
        let mut cfg = Config::default();
        cfg.wine_prefix = Some("/opt/custom-wine".to_string());
        assert_eq!(
            cfg.wine_prefix_path(),
            std::path::PathBuf::from("/opt/custom-wine")
        );
    }
}
