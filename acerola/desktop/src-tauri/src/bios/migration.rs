use std::path::{Path, PathBuf};

/// Returns the user's base application data directory (e.g., `%APPDATA%` on Windows,
/// `~/Library/Application Support` on macOS, `~/.local/share` on Linux).
pub fn get_base_data_dir() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        std::env::var_os("APPDATA").map(PathBuf::from).or_else(|| {
            std::env::var_os("USERPROFILE").map(|u| PathBuf::from(u).join("AppData\\Roaming"))
        })
    }
    #[cfg(target_os = "macos")]
    {
        std::env::var_os("HOME").map(|h| PathBuf::from(h).join("Library/Application Support"))
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))
    }
}

/// Migrates data from legacy directories (`br.acerola.reader` or `br.acerola.comic`) to the new directory (`Acerola Reader`)
/// if a legacy directory exists and the new directory does not yet contain the main database.
pub fn migrate_legacy_data_dir_if_needed() {
    let Some(base_dir) = get_base_data_dir() else {
        return;
    };

    let target_dir = base_dir.join("Acerola Reader");

    if target_dir.join("acerola.db").exists() {
        return;
    }

    let legacy_candidates = [base_dir.join("br.acerola.reader"), base_dir.join("br.acerola.comic")];

    let Some(legacy_dir) =
        legacy_candidates.into_iter().find(|p| p.join("acerola.db").exists() || p.exists())
    else {
        return;
    };

    tracing::info!(
        legacy = ?legacy_dir,
        target = ?target_dir,
        "[Migration] Migrating legacy application data"
    );

    if let Err(err) = std::fs::create_dir_all(&target_dir) {
        tracing::error!(
            error = %err,
            target = ?target_dir,
            "[Migration] Failed to create target directory"
        );
        return;
    }

    if let Err(err) = copy_dir_all(&legacy_dir, &target_dir) {
        tracing::error!(
            error = %err,
            "[Migration] Error occurred during legacy data migration"
        );
    } else {
        tracing::info!("[Migration] Legacy data migration completed successfully");
    }
}

fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let entry_type = entry.file_type()?;
        let dest_path = dst.join(entry.file_name());

        if entry_type.is_dir() {
            std::fs::create_dir_all(&dest_path)?;
            copy_dir_all(&entry.path(), &dest_path)?;
        } else if entry_type.is_file() && !dest_path.exists() {
            if let Err(copy_err) = std::fs::copy(entry.path(), &dest_path) {
                tracing::warn!(
                    error = %copy_err,
                    src = ?entry.path(),
                    dest = ?dest_path,
                    "[Migration] Failed to copy file during migration"
                );
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_base_data_dir() {
        let dir = get_base_data_dir();
        assert!(dir.is_some(), "Should resolve a base data directory");
    }

    #[test]
    fn test_copy_dir_all() {
        let temp_src = tempfile::tempdir().expect("Failed to create temporary source directory");
        let temp_dst =
            tempfile::tempdir().expect("Failed to create temporary destination directory");

        let file_a = temp_src.path().join("acerola.db");
        std::fs::write(&file_a, b"sqlite data").expect("Failed to write test file");

        let sub_dir = temp_src.path().join("logs");
        std::fs::create_dir_all(&sub_dir).expect("Failed to create subdirectory");
        let log_file = sub_dir.join("app.log");
        std::fs::write(&log_file, b"log data").expect("Failed to write test log");

        copy_dir_all(temp_src.path(), temp_dst.path()).expect("Failed to copy directory");

        assert!(temp_dst.path().join("acerola.db").exists());
        assert_eq!(std::fs::read(temp_dst.path().join("acerola.db")).unwrap(), b"sqlite data");
        assert!(temp_dst.path().join("logs").join("app.log").exists());
        assert_eq!(
            std::fs::read(temp_dst.path().join("logs").join("app.log")).unwrap(),
            b"log data"
        );
    }
}
