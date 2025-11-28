//! Plugin metadata and loading.
//!
//! Plugins are Lua scripts that can hook into Apollo's processing pipeline.
//! Each plugin should define a `plugin` table with metadata and hook functions.
//!
//! # Plugin Format
//!
//! ```lua
//! -- plugins/my_plugin.lua
//! local plugin = {
//!     name = "my_plugin",
//!     version = "1.0.0",
//!     description = "My awesome plugin",
//!     author = "Your Name",
//! }
//!
//! function plugin.on_import(track)
//!     -- Modify track before import
//!     if track.artist == "" then
//!         track.artist = "Unknown Artist"
//!     end
//!     return "continue" -- or "skip" or "abort"
//! end
//!
//! return plugin
//! ```

use crate::error::{Error, Result};
use crate::hooks::HookType;
use std::path::{Path, PathBuf};

/// Metadata about a loaded plugin.
#[derive(Debug, Clone)]
pub struct Plugin {
    /// Unique name of the plugin.
    pub name: String,
    /// Version string.
    pub version: String,
    /// Description of what the plugin does.
    pub description: String,
    /// Author of the plugin.
    pub author: Option<String>,
    /// Path to the plugin file.
    pub path: PathBuf,
    /// Which hooks this plugin provides.
    pub hooks: Vec<HookType>,
}

impl Plugin {
    /// Create a new plugin with the given metadata.
    #[must_use]
    pub const fn new(name: String, version: String, description: String, path: PathBuf) -> Self {
        Self {
            name,
            version,
            description,
            author: None,
            path,
            hooks: Vec::new(),
        }
    }

    /// Check if this plugin provides a specific hook.
    #[must_use]
    pub fn has_hook(&self, hook_type: HookType) -> bool {
        self.hooks.contains(&hook_type)
    }

    /// Get the Lua global table name for this plugin.
    ///
    /// This is used to store the plugin's functions in Lua's global namespace.
    #[must_use]
    pub fn lua_table_name(&self) -> String {
        format!("_plugin_{}", self.name.replace(['-', '.', ' '], "_"))
    }
}

impl std::fmt::Display for Plugin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} v{}", self.name, self.version)?;
        if let Some(author) = &self.author {
            write!(f, " by {author}")?;
        }
        Ok(())
    }
}

/// Load plugin metadata from a Lua file.
///
/// This parses the plugin table returned by the script to extract metadata.
///
/// # Errors
///
/// Returns an error if the file cannot be read or doesn't contain valid
/// plugin metadata.
pub fn load_plugin_metadata(path: &Path) -> Result<Plugin> {
    use std::fs;

    if !path.exists() {
        return Err(Error::PluginNotFound {
            path: path.to_path_buf(),
        });
    }

    let content = fs::read_to_string(path)?;

    // Extract basic metadata by parsing the Lua source
    // This is a simple approach that doesn't execute the Lua code
    let name = extract_string_field(&content, "name").ok_or_else(|| Error::InvalidMetadata {
        reason: "plugin must have a 'name' field".to_string(),
    })?;

    let version = extract_string_field(&content, "version").unwrap_or_else(|| "0.0.0".to_string());

    let description = extract_string_field(&content, "description").unwrap_or_default();

    let author = extract_string_field(&content, "author");

    let mut plugin = Plugin::new(name, version, description, path.to_path_buf());
    plugin.author = author;

    // Check which hooks are defined
    for hook_type in HookType::all() {
        let hook_pattern = format!("function plugin.{}", hook_type.lua_name());
        let hook_pattern_alt = format!("plugin.{} = function", hook_type.lua_name());
        if content.contains(&hook_pattern) || content.contains(&hook_pattern_alt) {
            plugin.hooks.push(*hook_type);
        }
    }

    Ok(plugin)
}

/// Extract a string field from Lua source code.
///
/// This is a simple regex-free parser that looks for patterns like:
/// - `name = "value"`
/// - `name = 'value'`
fn extract_string_field(content: &str, field: &str) -> Option<String> {
    // Look for patterns like: field = "value" or field = 'value'
    let patterns = [
        format!("{field} = \""),
        format!("{field} = '"),
        format!("{field}= \""),
        format!("{field}= '"),
        format!("{field} =\""),
        format!("{field} ='"),
    ];

    for pattern in &patterns {
        if let Some(start) = content.find(pattern) {
            let value_start = start + pattern.len();
            let quote_char = if pattern.ends_with('"') { '"' } else { '\'' };

            if let Some(end) = content[value_start..].find(quote_char) {
                return Some(content[value_start..value_start + end].to_string());
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_plugin_new() {
        let plugin = Plugin::new(
            "test_plugin".to_string(),
            "1.0.0".to_string(),
            "A test plugin".to_string(),
            PathBuf::from("/plugins/test.lua"),
        );

        assert_eq!(plugin.name, "test_plugin");
        assert_eq!(plugin.version, "1.0.0");
        assert_eq!(plugin.description, "A test plugin");
        assert!(plugin.hooks.is_empty());
    }

    #[test]
    fn test_plugin_lua_table_name() {
        let plugin = Plugin::new(
            "my-plugin.name".to_string(),
            "1.0.0".to_string(),
            "Test".to_string(),
            PathBuf::from("/plugins/test.lua"),
        );

        assert_eq!(plugin.lua_table_name(), "_plugin_my_plugin_name");
    }

    #[test]
    fn test_plugin_display() {
        let mut plugin = Plugin::new(
            "test".to_string(),
            "1.0.0".to_string(),
            "Test".to_string(),
            PathBuf::from("/plugins/test.lua"),
        );

        assert_eq!(plugin.to_string(), "test v1.0.0");

        plugin.author = Some("Author Name".to_string());
        assert_eq!(plugin.to_string(), "test v1.0.0 by Author Name");
    }

    #[test]
    fn test_extract_string_field() {
        let content = r#"
            local plugin = {
                name = "my_plugin",
                version = "1.0.0",
                description = 'A test plugin',
                author = "Test Author",
            }
        "#;

        assert_eq!(
            extract_string_field(content, "name"),
            Some("my_plugin".to_string())
        );
        assert_eq!(
            extract_string_field(content, "version"),
            Some("1.0.0".to_string())
        );
        assert_eq!(
            extract_string_field(content, "description"),
            Some("A test plugin".to_string())
        );
        assert_eq!(
            extract_string_field(content, "author"),
            Some("Test Author".to_string())
        );
        assert_eq!(extract_string_field(content, "nonexistent"), None);
    }

    #[test]
    fn test_load_plugin_metadata() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"
            local plugin = {{
                name = "test_plugin",
                version = "2.0.0",
                description = "Test description",
                author = "Test Author",
            }}

            function plugin.on_import(track)
                return "continue"
            end

            function plugin.on_update(track)
                return "continue"
            end

            return plugin
        "#
        )
        .unwrap();

        let plugin = load_plugin_metadata(file.path()).unwrap();

        assert_eq!(plugin.name, "test_plugin");
        assert_eq!(plugin.version, "2.0.0");
        assert_eq!(plugin.description, "Test description");
        assert_eq!(plugin.author, Some("Test Author".to_string()));
        assert!(plugin.has_hook(HookType::OnImport));
        assert!(plugin.has_hook(HookType::OnUpdate));
        assert!(!plugin.has_hook(HookType::PostImport));
    }

    #[test]
    fn test_load_plugin_metadata_not_found() {
        let result = load_plugin_metadata(Path::new("/nonexistent/plugin.lua"));
        assert!(matches!(result, Err(Error::PluginNotFound { .. })));
    }

    #[test]
    fn test_load_plugin_metadata_missing_name() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"
            local plugin = {{
                version = "1.0.0",
            }}
            return plugin
        "#
        )
        .unwrap();

        let result = load_plugin_metadata(file.path());
        assert!(matches!(result, Err(Error::InvalidMetadata { .. })));
    }
}
