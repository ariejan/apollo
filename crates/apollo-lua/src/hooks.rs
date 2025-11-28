//! Event hooks for the plugin system.
//!
//! Hooks allow plugins to respond to various events during the library
//! management process, such as importing new tracks or updating metadata.

use std::fmt;

/// The result of executing a hook.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum HookResult {
    /// Continue normal processing.
    #[default]
    Continue,
    /// Skip this item (don't import/update).
    Skip,
    /// Abort the entire operation.
    Abort {
        /// Reason for aborting.
        reason: String,
    },
}

impl HookResult {
    /// Create a new `Continue` result.
    #[must_use]
    pub const fn continue_() -> Self {
        Self::Continue
    }

    /// Create a new `Skip` result.
    #[must_use]
    pub const fn skip() -> Self {
        Self::Skip
    }

    /// Create a new `Abort` result with the given reason.
    #[must_use]
    pub fn abort(reason: impl Into<String>) -> Self {
        Self::Abort {
            reason: reason.into(),
        }
    }
}

impl fmt::Display for HookResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Continue => write!(f, "Continue"),
            Self::Skip => write!(f, "Skip"),
            Self::Abort { reason } => write!(f, "Abort: {reason}"),
        }
    }
}

/// Available hook types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HookType {
    /// Called before importing a track.
    OnImport,
    /// Called after a track is imported.
    PostImport,
    /// Called before updating track metadata.
    OnUpdate,
    /// Called after track metadata is updated.
    PostUpdate,
    /// Called before importing an album.
    OnAlbumImport,
    /// Called after an album is imported.
    PostAlbumImport,
    /// Called when the library is initialized.
    OnInit,
    /// Called when the library is closed.
    OnClose,
}

impl HookType {
    /// Get the Lua function name for this hook type.
    #[must_use]
    pub const fn lua_name(self) -> &'static str {
        match self {
            Self::OnImport => "on_import",
            Self::PostImport => "post_import",
            Self::OnUpdate => "on_update",
            Self::PostUpdate => "post_update",
            Self::OnAlbumImport => "on_album_import",
            Self::PostAlbumImport => "post_album_import",
            Self::OnInit => "on_init",
            Self::OnClose => "on_close",
        }
    }

    /// Get all hook types.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::OnImport,
            Self::PostImport,
            Self::OnUpdate,
            Self::PostUpdate,
            Self::OnAlbumImport,
            Self::PostAlbumImport,
            Self::OnInit,
            Self::OnClose,
        ]
    }
}

impl fmt::Display for HookType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.lua_name())
    }
}

/// Registry of hooks from loaded plugins.
#[derive(Debug, Default)]
pub struct Hooks {
    /// Registered hooks, keyed by hook type.
    hooks: Vec<(HookType, String)>,
}

impl Hooks {
    /// Create a new empty hooks registry.
    #[must_use]
    pub const fn new() -> Self {
        Self { hooks: Vec::new() }
    }

    /// Register a hook callback.
    ///
    /// The `callback_name` is the name of the Lua function to call,
    /// typically in the format `plugin_name.hook_name`.
    pub fn register(&mut self, hook_type: HookType, callback_name: String) {
        self.hooks.push((hook_type, callback_name));
    }

    /// Get all registered callbacks for a hook type.
    #[must_use]
    pub fn get(&self, hook_type: HookType) -> Vec<&str> {
        self.hooks
            .iter()
            .filter(|(ht, _)| *ht == hook_type)
            .map(|(_, name)| name.as_str())
            .collect()
    }

    /// Check if any hooks are registered for a hook type.
    #[must_use]
    pub fn has(&self, hook_type: HookType) -> bool {
        self.hooks.iter().any(|(ht, _)| *ht == hook_type)
    }

    /// Get the total number of registered hooks.
    #[must_use]
    #[allow(clippy::len_without_is_empty)] // is_empty is defined below
    pub const fn len(&self) -> usize {
        self.hooks.len()
    }

    /// Check if no hooks are registered.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.hooks.is_empty()
    }

    /// Clear all registered hooks.
    pub fn clear(&mut self) {
        self.hooks.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hook_result_default() {
        assert_eq!(HookResult::default(), HookResult::Continue);
    }

    #[test]
    fn test_hook_result_display() {
        assert_eq!(HookResult::Continue.to_string(), "Continue");
        assert_eq!(HookResult::Skip.to_string(), "Skip");
        assert_eq!(
            HookResult::Abort {
                reason: "error".to_string()
            }
            .to_string(),
            "Abort: error"
        );
    }

    #[test]
    fn test_hook_type_lua_names() {
        assert_eq!(HookType::OnImport.lua_name(), "on_import");
        assert_eq!(HookType::PostImport.lua_name(), "post_import");
        assert_eq!(HookType::OnUpdate.lua_name(), "on_update");
        assert_eq!(HookType::PostUpdate.lua_name(), "post_update");
    }

    #[test]
    fn test_hooks_registry() {
        let mut hooks = Hooks::new();

        assert!(hooks.is_empty());
        assert!(!hooks.has(HookType::OnImport));

        hooks.register(HookType::OnImport, "plugin1.on_import".to_string());
        hooks.register(HookType::OnImport, "plugin2.on_import".to_string());
        hooks.register(HookType::OnUpdate, "plugin1.on_update".to_string());

        assert_eq!(hooks.len(), 3);
        assert!(hooks.has(HookType::OnImport));
        assert!(hooks.has(HookType::OnUpdate));
        assert!(!hooks.has(HookType::PostImport));

        let import_hooks = hooks.get(HookType::OnImport);
        assert_eq!(import_hooks.len(), 2);
        assert!(import_hooks.contains(&"plugin1.on_import"));
        assert!(import_hooks.contains(&"plugin2.on_import"));

        hooks.clear();
        assert!(hooks.is_empty());
    }
}
