//! Configuration management for Apollo.
//!
//! Apollo uses a TOML configuration file to store settings. By default, the
//! configuration is stored at `~/.config/apollo/config.toml` on Unix systems
//! and `%APPDATA%\apollo\config.toml` on Windows.
//!
//! # Example Configuration
//!
//! ```toml
//! [library]
//! path = "~/.apollo/apollo.db"
//!
//! [import]
//! move_files = false
//! write_tags = true
//! copy_album_art = true
//!
//! [paths]
//! music_directory = "~/Music"
//! path_template = "$artist/$album/$track - $title"
//!
//! [musicbrainz]
//! enabled = true
//! auto_tag = false
//!
//! [acoustid]
//! api_key = ""
//!
//! [web]
//! host = "127.0.0.1"
//! port = 8337
//!
//! [plugins]
//! directory = "~/.config/apollo/plugins"
//! enabled = ["clean_tags", "skip_hidden"]
//! ```

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::error::Error;

/// Default configuration file name.
const CONFIG_FILE_NAME: &str = "config.toml";

/// Default library database file name.
const DEFAULT_DB_NAME: &str = "apollo.db";

/// Default library directory name (relative to home).
const DEFAULT_LIB_DIR: &str = ".apollo";

/// Default config directory name.
const DEFAULT_CONFIG_DIR: &str = "apollo";

/// Default web server port.
const DEFAULT_WEB_PORT: u16 = 8337;

/// Default web server host.
const DEFAULT_WEB_HOST: &str = "127.0.0.1";

/// Apollo configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct Config {
    /// Library settings.
    pub library: LibraryConfig,
    /// Import settings.
    pub import: ImportConfig,
    /// Path settings.
    pub paths: PathsConfig,
    /// [MusicBrainz](https://musicbrainz.org/) settings.
    pub musicbrainz: MusicBrainzConfig,
    /// [AcoustID](https://acoustid.org/) settings.
    pub acoustid: AcoustIdConfig,
    /// Web server settings.
    pub web: WebConfig,
    /// Plugin settings.
    pub plugins: PluginsConfig,
}

impl Config {
    /// Create a new configuration with default values.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the default configuration file path.
    ///
    /// Returns `~/.config/apollo/config.toml` on Unix or
    /// `%APPDATA%\apollo\config.toml` on Windows.
    #[must_use]
    pub fn default_path() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join(DEFAULT_CONFIG_DIR).join(CONFIG_FILE_NAME))
    }

    /// Get the default library database path.
    ///
    /// Returns `~/.apollo/apollo.db`.
    #[must_use]
    pub fn default_library_path() -> Option<PathBuf> {
        dirs::home_dir().map(|p| p.join(DEFAULT_LIB_DIR).join(DEFAULT_DB_NAME))
    }

    /// Load configuration from the default path.
    ///
    /// If the configuration file doesn't exist, returns the default configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration file exists but cannot be read or parsed.
    pub fn load() -> Result<Self, Error> {
        if let Some(path) = Self::default_path()
            && path.exists()
        {
            return Self::load_from(&path);
        }
        Ok(Self::default())
    }

    /// Load configuration from a specific path.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed.
    pub fn load_from(path: &Path) -> Result<Self, Error> {
        let content = std::fs::read_to_string(path).map_err(|e| Error::Config {
            message: format!("Failed to read config file: {e}"),
        })?;

        Self::from_toml(&content)
    }

    /// Parse configuration from a TOML string.
    ///
    /// # Errors
    ///
    /// Returns an error if the TOML is invalid.
    pub fn from_toml(content: &str) -> Result<Self, Error> {
        toml::from_str(content).map_err(|e| Error::Config {
            message: format!("Failed to parse config: {e}"),
        })
    }

    /// Serialize configuration to a TOML string.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails.
    pub fn to_toml(&self) -> Result<String, Error> {
        toml::to_string_pretty(self).map_err(|e| Error::Config {
            message: format!("Failed to serialize config: {e}"),
        })
    }

    /// Save configuration to the default path.
    ///
    /// Creates the parent directory if it doesn't exist.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written.
    pub fn save(&self) -> Result<(), Error> {
        let path = Self::default_path().ok_or_else(|| Error::Config {
            message: "Could not determine config directory".to_string(),
        })?;
        self.save_to(&path)
    }

    /// Save configuration to a specific path.
    ///
    /// Creates the parent directory if it doesn't exist.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written.
    pub fn save_to(&self, path: &Path) -> Result<(), Error> {
        // Create parent directory if needed
        if let Some(parent) = path.parent()
            && !parent.exists()
        {
            std::fs::create_dir_all(parent).map_err(|e| Error::Config {
                message: format!("Failed to create config directory: {e}"),
            })?;
        }

        let content = self.to_toml()?;
        std::fs::write(path, content).map_err(|e| Error::Config {
            message: format!("Failed to write config file: {e}"),
        })
    }

    /// Get the library database path, expanding `~` to home directory.
    #[must_use]
    pub fn library_path(&self) -> PathBuf {
        expand_tilde(&self.library.path)
    }

    /// Get the music directory path, expanding `~` to home directory.
    #[must_use]
    pub fn music_directory(&self) -> Option<PathBuf> {
        self.paths.music_directory.as_ref().map(|p| expand_tilde(p))
    }

    /// Get the plugins directory path, expanding `~` to home directory.
    #[must_use]
    pub fn plugins_directory(&self) -> PathBuf {
        expand_tilde(&self.plugins.directory)
    }
}

/// Library configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct LibraryConfig {
    /// Path to the library database file.
    pub path: PathBuf,
}

impl Default for LibraryConfig {
    fn default() -> Self {
        Self {
            path: PathBuf::from(format!("~/{DEFAULT_LIB_DIR}/{DEFAULT_DB_NAME}")),
        }
    }
}

/// Import configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
#[allow(clippy::struct_excessive_bools)]
pub struct ImportConfig {
    /// Move files instead of copying during import.
    pub move_files: bool,
    /// Write metadata tags to imported files.
    pub write_tags: bool,
    /// Copy album art files.
    pub copy_album_art: bool,
    /// Automatically create albums from grouped tracks.
    pub auto_create_albums: bool,
    /// Compute and store file hashes for deduplication.
    pub compute_hashes: bool,
}

impl Default for ImportConfig {
    fn default() -> Self {
        Self {
            move_files: false,
            write_tags: true,
            copy_album_art: true,
            auto_create_albums: true,
            compute_hashes: true,
        }
    }
}

/// Path configuration for file organization.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct PathsConfig {
    /// Base music directory.
    pub music_directory: Option<PathBuf>,
    /// Template for organizing files.
    /// Supports: $artist, $album, $track, $title, $year, $genre
    pub path_template: String,
}

impl Default for PathsConfig {
    fn default() -> Self {
        Self {
            music_directory: None,
            path_template: "$artist/$album/$track - $title".to_string(),
        }
    }
}

/// [MusicBrainz](https://musicbrainz.org/) integration configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct MusicBrainzConfig {
    /// Enable [MusicBrainz](https://musicbrainz.org/) integration.
    pub enabled: bool,
    /// Automatically fetch and apply tags from [MusicBrainz](https://musicbrainz.org/).
    pub auto_tag: bool,
    /// Application name for API requests.
    pub app_name: String,
    /// Application version for API requests.
    pub app_version: String,
    /// Contact email for API requests (recommended by [MusicBrainz](https://musicbrainz.org/)).
    pub contact_email: String,
}

impl Default for MusicBrainzConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            auto_tag: false,
            app_name: "Apollo".to_string(),
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            contact_email: String::new(),
        }
    }
}

/// [AcoustID](https://acoustid.org/) fingerprinting configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct AcoustIdConfig {
    /// Enable [AcoustID](https://acoustid.org/) fingerprinting.
    pub enabled: bool,
    /// [AcoustID](https://acoustid.org/) API key (get one at <https://acoustid.org/new-application>).
    pub api_key: String,
    /// Automatically lookup fingerprints during import.
    pub auto_lookup: bool,
}

impl Default for AcoustIdConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            api_key: String::new(),
            auto_lookup: false,
        }
    }
}

/// Web server configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct WebConfig {
    /// Host address to bind to.
    pub host: String,
    /// Port to listen on.
    pub port: u16,
    /// Enable Swagger UI.
    pub swagger_ui: bool,
}

impl Default for WebConfig {
    fn default() -> Self {
        Self {
            host: DEFAULT_WEB_HOST.to_string(),
            port: DEFAULT_WEB_PORT,
            swagger_ui: true,
        }
    }
}

/// Plugin configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct PluginsConfig {
    /// Directory containing Lua plugins.
    pub directory: PathBuf,
    /// List of enabled plugins (by name, without .lua extension).
    pub enabled: Vec<String>,
}

impl Default for PluginsConfig {
    fn default() -> Self {
        let dir = dirs::config_dir().map_or_else(
            || PathBuf::from("~/.config/apollo/plugins"),
            |p| p.join(DEFAULT_CONFIG_DIR).join("plugins"),
        );

        Self {
            directory: dir,
            enabled: Vec::new(),
        }
    }
}

/// Expand `~` to the home directory in a path.
fn expand_tilde(path: &Path) -> PathBuf {
    let path_str = path.to_string_lossy();
    if let Some(stripped) = path_str.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(stripped);
        }
    } else if path_str == "~"
        && let Some(home) = dirs::home_dir()
    {
        return home;
    }
    path.to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.musicbrainz.enabled);
        assert!(!config.musicbrainz.auto_tag);
        assert!(config.acoustid.enabled);
        assert!(config.acoustid.api_key.is_empty());
        assert_eq!(config.web.port, 8337);
    }

    #[test]
    fn test_config_roundtrip() {
        let config = Config::default();
        let toml = config.to_toml().unwrap();
        let parsed = Config::from_toml(&toml).unwrap();
        assert_eq!(config, parsed);
    }

    #[test]
    fn test_config_from_toml() {
        let toml = r#"
[library]
path = "/custom/path/apollo.db"

[web]
port = 9000
host = "0.0.0.0"

[acoustid]
api_key = "my-api-key"
enabled = true
"#;
        let config = Config::from_toml(toml).unwrap();
        assert_eq!(config.library.path, PathBuf::from("/custom/path/apollo.db"));
        assert_eq!(config.web.port, 9000);
        assert_eq!(config.web.host, "0.0.0.0");
        assert_eq!(config.acoustid.api_key, "my-api-key");
    }

    #[test]
    fn test_expand_tilde() {
        let home = dirs::home_dir();

        if let Some(ref h) = home {
            assert_eq!(expand_tilde(Path::new("~/test")), h.join("test"));
            assert_eq!(expand_tilde(Path::new("~")), h.clone());
        }

        // Non-tilde paths should be unchanged
        assert_eq!(
            expand_tilde(Path::new("/absolute/path")),
            PathBuf::from("/absolute/path")
        );
        assert_eq!(
            expand_tilde(Path::new("relative/path")),
            PathBuf::from("relative/path")
        );
    }

    #[test]
    fn test_partial_config() {
        // Only specify some values, rest should use defaults
        let toml = r#"
[web]
port = 3000
"#;
        let config = Config::from_toml(toml).unwrap();
        assert_eq!(config.web.port, 3000);
        assert_eq!(config.web.host, DEFAULT_WEB_HOST); // Default
        assert!(config.musicbrainz.enabled); // Default
    }

    #[test]
    fn test_plugins_config() {
        let toml = r#"
[plugins]
directory = "~/my-plugins"
enabled = ["clean_tags", "skip_hidden", "custom_plugin"]
"#;
        let config = Config::from_toml(toml).unwrap();
        assert_eq!(config.plugins.directory, PathBuf::from("~/my-plugins"));
        assert_eq!(config.plugins.enabled.len(), 3);
        assert!(config.plugins.enabled.contains(&"clean_tags".to_string()));
    }

    #[test]
    fn test_default_paths() {
        // These should return Some on most systems
        let config_path = Config::default_path();
        let lib_path = Config::default_library_path();

        // Just check that they work (values depend on system)
        if let Some(path) = config_path {
            assert!(path.to_string_lossy().contains("apollo"));
        }
        if let Some(path) = lib_path {
            assert!(path.to_string_lossy().contains("apollo"));
        }
    }

    #[test]
    fn test_import_config() {
        let toml = r#"
[import]
move_files = true
write_tags = false
copy_album_art = false
"#;
        let config = Config::from_toml(toml).unwrap();
        assert!(config.import.move_files);
        assert!(!config.import.write_tags);
        assert!(!config.import.copy_album_art);
        assert!(config.import.auto_create_albums); // Default
    }
}
