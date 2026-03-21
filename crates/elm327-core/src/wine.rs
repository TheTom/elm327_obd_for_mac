use std::path::{Path, PathBuf};

use crate::error::{BridgeError, Result};

/// Create Wine COM port symlink.
/// Maps e.g. `~/.wine/dosdevices/com3` -> `/dev/ttys003`
///
/// # Arguments
/// * `wine_prefix` - Path to the Wine prefix (e.g. `~/.wine`)
/// * `com_port` - COM port name (e.g. "COM3"), will be lowercased
/// * `pty_device` - Path to the PTY device to link to
///
/// # Returns
/// The path of the created symlink.
pub fn create_com_symlink(
    wine_prefix: &Path,
    com_port: &str,
    pty_device: &Path,
) -> Result<PathBuf> {
    let dosdevices = wine_prefix.join("dosdevices");

    if !dosdevices.exists() {
        return Err(BridgeError::WinePrefix {
            path: wine_prefix.to_path_buf(),
        });
    }

    // Wine expects lowercase: com3, not COM3
    let link_name = com_port.to_lowercase();
    let link_path = dosdevices.join(&link_name);

    // Remove existing symlink if present
    if link_path.exists() || link_path.symlink_metadata().is_ok() {
        std::fs::remove_file(&link_path).map_err(BridgeError::Io)?;
        log::debug!("Removed existing symlink: {}", link_path.display());
    }

    // Create symlink
    std::os::unix::fs::symlink(pty_device, &link_path).map_err(BridgeError::Io)?;

    log::info!(
        "Created COM symlink: {} -> {}",
        link_path.display(),
        pty_device.display()
    );

    Ok(link_path)
}

/// Remove COM port symlink from the Wine prefix.
pub fn remove_com_symlink(wine_prefix: &Path, com_port: &str) -> Result<()> {
    let link_path = wine_prefix
        .join("dosdevices")
        .join(com_port.to_lowercase());

    if link_path.symlink_metadata().is_ok() {
        std::fs::remove_file(&link_path).map_err(BridgeError::Io)?;
        log::info!("Removed COM symlink: {}", link_path.display());
    }

    Ok(())
}

/// Validate that a Wine prefix exists and has a dosdevices directory.
pub fn validate_wine_prefix(prefix: &Path) -> Result<()> {
    let dosdevices = prefix.join("dosdevices");
    if !dosdevices.exists() {
        return Err(BridgeError::WinePrefix {
            path: prefix.to_path_buf(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    /// Helper: create a temp dir that looks like a Wine prefix with dosdevices/
    fn make_fake_prefix() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "elm327_wine_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(dir.join("dosdevices")).unwrap();
        dir
    }

    #[test]
    fn test_create_symlink() {
        let prefix = make_fake_prefix();
        let pty = PathBuf::from("/dev/ttys003");

        let result = create_com_symlink(&prefix, "com3", &pty).unwrap();

        assert!(result.symlink_metadata().is_ok(), "symlink should exist");
        let target = fs::read_link(&result).unwrap();
        assert_eq!(target, pty);

        // Cleanup
        fs::remove_dir_all(&prefix).unwrap();
    }

    #[test]
    fn test_remove_symlink() {
        let prefix = make_fake_prefix();
        let pty = PathBuf::from("/dev/ttys003");

        // Create then remove
        let link = create_com_symlink(&prefix, "com5", &pty).unwrap();
        assert!(link.symlink_metadata().is_ok());

        remove_com_symlink(&prefix, "com5").unwrap();
        assert!(
            link.symlink_metadata().is_err(),
            "symlink should be gone after removal"
        );

        // Cleanup
        fs::remove_dir_all(&prefix).unwrap();
    }

    #[test]
    fn test_validate_prefix_missing() {
        let bogus = std::env::temp_dir().join("elm327_wine_test_nonexistent_prefix");
        let result = validate_wine_prefix(&bogus);
        assert!(result.is_err());
        match result.unwrap_err() {
            BridgeError::WinePrefix { path } => assert_eq!(path, bogus),
            other => panic!("Expected WinePrefix error, got: {:?}", other),
        }
    }

    #[test]
    fn test_com_port_lowercase() {
        let prefix = make_fake_prefix();
        let pty = PathBuf::from("/dev/ttys007");

        let result = create_com_symlink(&prefix, "COM3", &pty).unwrap();

        // Should be lowercase in the path
        assert!(
            result.to_string_lossy().contains("com3"),
            "COM3 should become com3 in path, got: {}",
            result.display()
        );
        assert!(!result.to_string_lossy().contains("COM3"));

        // Cleanup
        fs::remove_dir_all(&prefix).unwrap();
    }
}
