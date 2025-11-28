//! # Apollo Lua
//!
//! Lua scripting and plugin support for Apollo.
//!
//! This crate provides a Lua 5.4 runtime for extending Apollo with custom
//! scripts and plugins. It exposes core types like [`Track`] and [`Album`]
//! to Lua, allowing users to write custom import hooks, metadata processors,
//! and other extensions.
//!
//! # Example
//!
//! ```no_run
//! use apollo_lua::LuaRuntime;
//! use apollo_core::Track;
//! use std::path::PathBuf;
//! use std::time::Duration;
//!
//! # fn example() -> Result<(), apollo_lua::Error> {
//! // Create a new Lua runtime
//! let mut runtime = LuaRuntime::new()?;
//!
//! // Load a plugin
//! runtime.load_plugin("plugins/my_plugin.lua")?;
//!
//! // Create a track
//! let mut track = Track::new(
//!     PathBuf::from("/music/song.mp3"),
//!     "My Song".to_string(),
//!     "Artist".to_string(),
//!     Duration::from_secs(180),
//! );
//!
//! // Run the on_import hook
//! runtime.run_on_import(&mut track)?;
//! # Ok(())
//! # }
//! ```

mod bindings;
mod error;
mod hooks;
mod plugin;
mod runtime;

pub use error::Error;
pub use hooks::{HookResult, Hooks};
pub use plugin::Plugin;
pub use runtime::LuaRuntime;
