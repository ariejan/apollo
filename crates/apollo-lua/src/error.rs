//! Error types for the Lua runtime.

use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur during Lua operations.
#[derive(Debug, Error)]
pub enum Error {
    /// Error from the Lua runtime.
    #[error("Lua error: {0}")]
    Lua(#[from] mlua::Error),

    /// Plugin file not found.
    #[error("Plugin not found: {path}")]
    PluginNotFound {
        /// Path to the missing plugin.
        path: PathBuf,
    },

    /// Error loading a plugin.
    #[error("Failed to load plugin '{name}': {reason}")]
    PluginLoad {
        /// Name of the plugin.
        name: String,
        /// Reason for the failure.
        reason: String,
    },

    /// Hook execution failed.
    #[error("Hook '{hook}' failed: {reason}")]
    HookFailed {
        /// Name of the hook.
        hook: String,
        /// Reason for the failure.
        reason: String,
    },

    /// Invalid plugin metadata.
    #[error("Invalid plugin metadata: {reason}")]
    InvalidMetadata {
        /// Reason the metadata is invalid.
        reason: String,
    },

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// A `Result` type alias using our `Error` type.
pub type Result<T> = std::result::Result<T, Error>;
