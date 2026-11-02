//! @file presets.rs
//! @brief Profile preset manager for saving, loading, listing, and deleting audio routing graph configurations.
//! @details Manages JSON persistence of audio patchbay graph topologies to local storage directories.
//! Supports sanitization of illegal filesystem characters, robust directory scanning, and graceful
//! error handling for corrupt or malformed preset files.

use crate::commands::dto::AudioGraphDto;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// @brief Sanitizes a profile name into a safe filesystem filename stem.
/// @param name Raw profile name string.
/// @return Sanitized string safe for use as a filename on Windows and other operating systems.
fn sanitize_filename(name: &str) -> String {
    let sanitized: String = name
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c if (c as u32) < 32 => '_',
            c => c,
        })
        .collect();
    let trimmed = sanitized.trim();
    if trimmed.is_empty() {
        "profile".to_string()
    } else {
        trimmed.to_string()
    }
}

/// @brief Determines the default presets storage directory based on environment variables or local app data.
/// @return PathBuf representing the presets storage directory.
fn default_presets_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("DYNO_PRESETS_DIR") {
        return PathBuf::from(dir);
    }
    if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        return PathBuf::from(local_app_data)
            .join("DynoVirtualAudioMixer")
            .join("presets");
    }
    PathBuf::from("presets")
}

/// @brief Data structure representing a saved audio routing graph preset profile.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProfilePreset {
    /// @brief Unique identifier of the profile preset.
    pub id: String,
    /// @brief Human-readable name of the profile preset.
    pub name: String,
    /// @brief Timestamp string when the profile preset was created.
    pub created_at: String,
    /// @brief Snapshot of the audio routing patchbay graph.
    pub graph: AudioGraphDto,
}

impl ProfilePreset {
    /// @brief Constructs a new ProfilePreset instance.
    /// @param id Unique identifier string for the preset.
    /// @param name Friendly user-facing name for the preset.
    /// @param created_at Timestamp of creation.
    /// @param graph AudioGraphDto containing nodes and connections.
    /// @return Initialized ProfilePreset instance.
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        created_at: impl Into<String>,
        graph: AudioGraphDto,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            created_at: created_at.into(),
            graph,
        }
    }
}

/// @brief Manager responsible for reading, writing, and deleting profile preset files on disk.
#[derive(Debug, Clone)]
pub struct ProfileManager {
    /// @brief Base directory where preset JSON files are persisted.
    base_dir: PathBuf,
}

impl Default for ProfileManager {
    /// @brief Constructs a default ProfileManager pointing to the system local app data directory.
    /// @return Initialized ProfileManager.
    fn default() -> Self {
        Self::new(default_presets_dir())
    }
}

impl ProfileManager {
    /// @brief Constructs a new ProfileManager pointing to the specified storage directory.
    /// @param base_dir Target filesystem directory path for storing preset JSON files.
    /// @return Initialized ProfileManager instance.
    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    /// @brief Returns a reference to the active presets storage directory path.
    /// @return Path slice of the base directory.
    pub fn base_dir(&self) -> &Path {
        &self.base_dir
    }

    /// @brief Resolves candidate direct file paths for a preset by its name.
    /// @param name Raw profile name or filename string.
    /// @return Vector of PathBuf candidate locations.
    fn resolve_candidate_paths(&self, name: &str) -> Vec<PathBuf> {
        let safe_name = sanitize_filename(name);
        let direct_path = self.base_dir.join(format!("{safe_name}.json"));
        let raw_path = if name.ends_with(".json") {
            self.base_dir.join(name)
        } else {
            self.base_dir.join(format!("{name}.json"))
        };
        if direct_path == raw_path {
            vec![direct_path]
        } else {
            vec![direct_path, raw_path]
        }
    }

    /// @brief Scans the base directory and returns all valid JSON file paths.
    /// @return `Ok(Vec<PathBuf>)` containing JSON paths, or `Err(String)` on I/O error.
    fn scan_json_files(&self) -> Result<Vec<PathBuf>, String> {
        if !self.base_dir.is_dir() {
            return Ok(Vec::new());
        }
        let entries = fs::read_dir(&self.base_dir).map_err(|err| {
            format!(
                "Failed to scan presets directory '{}': {err}",
                self.base_dir.display()
            )
        })?;
        let mut paths = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
                paths.push(path);
            }
        }
        Ok(paths)
    }

    /// @brief Searches scanned preset files for one matching the specified name or identifier.
    /// @param name Profile name or unique identifier.
    /// @return `Ok(Some((PathBuf, ProfilePreset)))` if found, `Ok(None)` if not found, or `Err(String)`.
    fn find_scanned_preset(&self, name: &str) -> Result<Option<(PathBuf, ProfilePreset)>, String> {
        let json_paths = self.scan_json_files()?;
        for path in json_paths {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(preset) = serde_json::from_str::<ProfilePreset>(&content) {
                    if preset.name == name || preset.id == name {
                        return Ok(Some((path, preset)));
                    }
                }
            }
        }
        Ok(None)
    }

    /// @brief Saves a profile preset to disk as a pretty-printed JSON file.
    /// @param preset The ProfilePreset to persist.
    /// @return `Ok(String)` containing the saved preset name, or `Err(String)` describing the I/O error.
    pub fn save_profile(&self, preset: &ProfilePreset) -> Result<String, String> {
        if let Err(err) = fs::create_dir_all(&self.base_dir) {
            return Err(format!(
                "Failed to create presets directory '{}': {err}",
                self.base_dir.display()
            ));
        }

        let filename = format!("{}.json", sanitize_filename(&preset.name));
        let target_path = self.base_dir.join(filename);

        let json_data = serde_json::to_string_pretty(preset)
            .map_err(|err| format!("Failed to serialize preset '{}': {err}", preset.name))?;

        fs::write(&target_path, json_data).map_err(|err| {
            format!(
                "Failed to write preset file '{}': {err}",
                target_path.display()
            )
        })?;

        Ok(preset.name.clone())
    }

    /// @brief Loads a profile preset by its name, sanitized filename, or identifier.
    /// @param name Profile name or unique identifier to locate and deserialize.
    /// @return `Ok(ProfilePreset)` on successful load, or `Err(String)` if missing or corrupt.
    pub fn load_profile(&self, name: &str) -> Result<ProfilePreset, String> {
        for candidate in self.resolve_candidate_paths(name) {
            if candidate.is_file() {
                let content = fs::read_to_string(&candidate).map_err(|err| {
                    format!(
                        "Failed to read preset file '{}': {err}",
                        candidate.display()
                    )
                })?;
                return serde_json::from_str(&content).map_err(|err| {
                    format!(
                        "Corrupt or malformed preset file '{}': {err}",
                        candidate.display()
                    )
                });
            }
        }

        if let Some((_, preset)) = self.find_scanned_preset(name)? {
            return Ok(preset);
        }

        Err(format!("Profile preset '{name}' was not found."))
    }

    /// @brief Lists all valid profile preset names discovered in the presets directory.
    /// @return `Ok(Vec<String>)` containing sorted profile names, or `Err(String)` on filesystem error.
    pub fn list_profiles(&self) -> Result<Vec<String>, String> {
        let json_paths = self.scan_json_files()?;
        let mut names = Vec::new();

        for path in json_paths {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(preset) = serde_json::from_str::<ProfilePreset>(&content) {
                    names.push(preset.name);
                }
            }
        }

        names.sort();
        names.dedup();
        Ok(names)
    }

    /// @brief Deletes a profile preset file matching the given name or identifier.
    /// @param name Profile name or unique identifier to remove from disk.
    /// @return `Ok(true)` if deleted, `Ok(false)` if not found, or `Err(String)` on I/O error.
    pub fn delete_profile(&self, name: &str) -> Result<bool, String> {
        for candidate in self.resolve_candidate_paths(name) {
            if candidate.is_file() {
                fs::remove_file(&candidate).map_err(|err| {
                    format!(
                        "Failed to delete preset file '{}': {err}",
                        candidate.display()
                    )
                })?;
                return Ok(true);
            }
        }

        if let Some((path, _)) = self.find_scanned_preset(name)? {
            fs::remove_file(&path).map_err(|err| {
                format!(
                    "Failed to delete preset file '{}': {err}",
                    path.display()
                )
            })?;
            return Ok(true);
        }

        Ok(false)
    }
}
