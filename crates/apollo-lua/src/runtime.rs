//! The Lua runtime for executing plugins and hooks.

use crate::bindings::{LuaAlbum, LuaTrack, register_apollo_module};
use crate::error::{Error, Result};
use crate::hooks::{HookResult, HookType, Hooks};
use crate::plugin::{Plugin, load_plugin_metadata};
use apollo_core::{Album, Track};
use mlua::{Function, Lua, Value};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tracing::{debug, info, warn};

/// The Lua runtime for Apollo plugins.
///
/// This manages the Lua VM, loaded plugins, and hook execution.
pub struct LuaRuntime {
    /// The Lua VM instance.
    lua: Lua,
    /// Loaded plugins.
    plugins: HashMap<String, Plugin>,
    /// Registered hooks.
    hooks: Hooks,
}

impl LuaRuntime {
    /// Create a new Lua runtime.
    ///
    /// # Errors
    ///
    /// Returns an error if the Lua VM cannot be created or initialized.
    pub fn new() -> Result<Self> {
        let lua = Lua::new();

        // Register the apollo module
        register_apollo_module(&lua)?;

        // Set up the plugins table
        lua.globals().set("_plugins", lua.create_table()?)?;

        Ok(Self {
            lua,
            plugins: HashMap::new(),
            hooks: Hooks::new(),
        })
    }

    /// Load a plugin from a file.
    ///
    /// The plugin script should return a table with metadata and hook functions.
    ///
    /// # Errors
    ///
    /// Returns an error if the plugin cannot be loaded or is invalid.
    ///
    /// # Panics
    ///
    /// Panics if the plugin was inserted but cannot be retrieved (should never happen).
    pub fn load_plugin<P: AsRef<Path>>(&mut self, path: P) -> Result<&Plugin> {
        let path = path.as_ref();

        // Load metadata first (without executing)
        let plugin = load_plugin_metadata(path)?;
        let plugin_name = plugin.name.clone();

        info!("Loading plugin: {} v{}", plugin.name, plugin.version);

        // Read and execute the plugin script
        let script = fs::read_to_string(path)?;
        let plugin_table: mlua::Table =
            self.lua
                .load(&script)
                .eval()
                .map_err(|e| Error::PluginLoad {
                    name: plugin_name.clone(),
                    reason: e.to_string(),
                })?;

        // Store the plugin table in globals
        let table_name = plugin.lua_table_name();
        self.lua.globals().set(table_name.as_str(), plugin_table)?;

        // Register hooks
        for hook_type in &plugin.hooks {
            let callback_name = format!("{}.{}", table_name, hook_type.lua_name());
            self.hooks.register(*hook_type, callback_name);
            debug!("Registered hook: {} for {}", hook_type, plugin.name);
        }

        // Store the plugin
        self.plugins.insert(plugin_name.clone(), plugin);

        Ok(self.plugins.get(&plugin_name).expect("just inserted"))
    }

    /// Load all plugins from a directory.
    ///
    /// Loads all `.lua` files in the specified directory.
    ///
    /// # Errors
    ///
    /// Returns errors for individual plugins but continues loading others.
    pub fn load_plugins_from_directory<P: AsRef<Path>>(&mut self, dir: P) -> Vec<Result<String>> {
        let dir = dir.as_ref();
        let mut results = Vec::new();

        let entries = match fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(e) => {
                results.push(Err(Error::Io(e)));
                return results;
            }
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "lua") {
                match self.load_plugin(&path) {
                    Ok(plugin) => {
                        results.push(Ok(plugin.name.clone()));
                    }
                    Err(e) => {
                        warn!("Failed to load plugin {:?}: {}", path, e);
                        results.push(Err(e));
                    }
                }
            }
        }

        results
    }

    /// Get a loaded plugin by name.
    #[must_use]
    pub fn get_plugin(&self, name: &str) -> Option<&Plugin> {
        self.plugins.get(name)
    }

    /// Get all loaded plugins.
    #[must_use]
    pub fn plugins(&self) -> Vec<&Plugin> {
        self.plugins.values().collect()
    }

    /// Check if any hooks are registered for a hook type.
    #[must_use]
    pub fn has_hooks(&self, hook_type: HookType) -> bool {
        self.hooks.has(hook_type)
    }

    /// Run the `on_import` hook for a track.
    ///
    /// All registered `on_import` handlers are called in order.
    /// If any handler returns "skip" or "abort", processing stops.
    ///
    /// # Errors
    ///
    /// Returns an error if a hook fails or returns "abort".
    pub fn run_on_import(&self, track: &mut Track) -> Result<HookResult> {
        self.run_track_hook(HookType::OnImport, track)
    }

    /// Run the `post_import` hook for a track.
    ///
    /// # Errors
    ///
    /// Returns an error if a hook fails.
    pub fn run_post_import(&self, track: &Track) -> Result<HookResult> {
        // post_import doesn't modify the track
        let mut track_copy = track.clone();
        self.run_track_hook(HookType::PostImport, &mut track_copy)
    }

    /// Run the `on_update` hook for a track.
    ///
    /// # Errors
    ///
    /// Returns an error if a hook fails or returns "abort".
    pub fn run_on_update(&self, track: &mut Track) -> Result<HookResult> {
        self.run_track_hook(HookType::OnUpdate, track)
    }

    /// Run the `post_update` hook for a track.
    ///
    /// # Errors
    ///
    /// Returns an error if a hook fails.
    pub fn run_post_update(&self, track: &Track) -> Result<HookResult> {
        let mut track_copy = track.clone();
        self.run_track_hook(HookType::PostUpdate, &mut track_copy)
    }

    /// Run the `on_album_import` hook for an album.
    ///
    /// # Errors
    ///
    /// Returns an error if a hook fails or returns "abort".
    pub fn run_on_album_import(&self, album: &mut Album) -> Result<HookResult> {
        self.run_album_hook(HookType::OnAlbumImport, album)
    }

    /// Run the `post_album_import` hook for an album.
    ///
    /// # Errors
    ///
    /// Returns an error if a hook fails.
    pub fn run_post_album_import(&self, album: &Album) -> Result<HookResult> {
        let mut album_copy = album.clone();
        self.run_album_hook(HookType::PostAlbumImport, &mut album_copy)
    }

    /// Run the `on_init` hook.
    ///
    /// # Errors
    ///
    /// Returns an error if a hook fails.
    pub fn run_on_init(&self) -> Result<()> {
        for callback in self.hooks.get(HookType::OnInit) {
            let func = self.get_callback_function(callback)?;
            func.call::<_, ()>(()).map_err(|e| Error::HookFailed {
                hook: "on_init".to_string(),
                reason: e.to_string(),
            })?;
        }
        Ok(())
    }

    /// Run the `on_close` hook.
    ///
    /// # Errors
    ///
    /// Returns an error if a hook fails.
    pub fn run_on_close(&self) -> Result<()> {
        for callback in self.hooks.get(HookType::OnClose) {
            let func = self.get_callback_function(callback)?;
            func.call::<_, ()>(()).map_err(|e| Error::HookFailed {
                hook: "on_close".to_string(),
                reason: e.to_string(),
            })?;
        }
        Ok(())
    }

    /// Execute arbitrary Lua code.
    ///
    /// This is useful for testing or running one-off scripts.
    ///
    /// # Errors
    ///
    /// Returns an error if the code fails to execute.
    pub fn exec(&self, code: &str) -> Result<()> {
        self.lua.load(code).exec()?;
        Ok(())
    }

    /// Evaluate Lua code and return the result as a string.
    ///
    /// # Errors
    ///
    /// Returns an error if the code fails to execute.
    pub fn eval<T: for<'a> mlua::FromLua<'a>>(&self, code: &str) -> Result<T> {
        let result = self.lua.load(code).eval()?;
        Ok(result)
    }

    /// Run a hook that operates on a track.
    fn run_track_hook(&self, hook_type: HookType, track: &mut Track) -> Result<HookResult> {
        let callbacks = self.hooks.get(hook_type);
        if callbacks.is_empty() {
            return Ok(HookResult::Continue);
        }

        // Create a shared Lua track wrapper
        let lua_track = LuaTrack::new(track.clone());
        self.lua
            .globals()
            .set("_current_track", lua_track.clone())?;

        for callback in callbacks {
            let func = self.get_callback_function(callback)?;

            let result: Value = func
                .call(lua_track.clone())
                .map_err(|e| Error::HookFailed {
                    hook: hook_type.to_string(),
                    reason: e.to_string(),
                })?;

            let hook_result = parse_hook_result(&result);

            match &hook_result {
                HookResult::Skip => {
                    debug!("Hook {} returned skip", callback);
                    return Ok(hook_result);
                }
                HookResult::Abort { reason } => {
                    warn!("Hook {} aborted: {}", callback, reason);
                    return Ok(hook_result);
                }
                HookResult::Continue => {}
            }
        }

        // Copy changes back to the original track
        *track = lua_track.get();

        Ok(HookResult::Continue)
    }

    /// Run a hook that operates on an album.
    fn run_album_hook(&self, hook_type: HookType, album: &mut Album) -> Result<HookResult> {
        let callbacks = self.hooks.get(hook_type);
        if callbacks.is_empty() {
            return Ok(HookResult::Continue);
        }

        let lua_album = LuaAlbum::new(album.clone());
        self.lua
            .globals()
            .set("_current_album", lua_album.clone())?;

        for callback in callbacks {
            let func = self.get_callback_function(callback)?;

            let result: Value = func
                .call(lua_album.clone())
                .map_err(|e| Error::HookFailed {
                    hook: hook_type.to_string(),
                    reason: e.to_string(),
                })?;

            let hook_result = parse_hook_result(&result);

            match &hook_result {
                HookResult::Skip => {
                    debug!("Hook {} returned skip", callback);
                    return Ok(hook_result);
                }
                HookResult::Abort { reason } => {
                    warn!("Hook {} aborted: {}", callback, reason);
                    return Ok(hook_result);
                }
                HookResult::Continue => {}
            }
        }

        // Copy changes back to the original album
        *album = lua_album.get();

        Ok(HookResult::Continue)
    }

    /// Get a callback function from its name (e.g., `_plugin_foo.on_import`).
    fn get_callback_function(&self, callback: &str) -> Result<Function<'_>> {
        let parts: Vec<&str> = callback.split('.').collect();
        if parts.len() != 2 {
            return Err(Error::HookFailed {
                hook: callback.to_string(),
                reason: "invalid callback name".to_string(),
            });
        }

        let table: mlua::Table =
            self.lua
                .globals()
                .get(parts[0])
                .map_err(|e| Error::HookFailed {
                    hook: callback.to_string(),
                    reason: format!("plugin table not found: {e}"),
                })?;

        let func: Function = table.get(parts[1]).map_err(|e| Error::HookFailed {
            hook: callback.to_string(),
            reason: format!("hook function not found: {e}"),
        })?;

        Ok(func)
    }
}

/// Parse a Lua value into a `HookResult`.
#[allow(clippy::match_same_arms)] // Nil is explicitly handled for clarity
fn parse_hook_result(value: &Value) -> HookResult {
    match value {
        Value::Nil => HookResult::Continue,
        Value::String(s) => match s.to_str().unwrap_or("continue").to_lowercase().as_str() {
            "skip" => HookResult::Skip,
            "abort" => HookResult::Abort {
                reason: "plugin requested abort".to_string(),
            },
            _ => HookResult::Continue,
        },
        Value::Table(t) => {
            // Check for { result = "abort", reason = "..." } format
            if let Ok(result) = t.get::<_, String>("result") {
                if result.to_lowercase() == "abort" {
                    let reason = t
                        .get::<_, String>("reason")
                        .unwrap_or_else(|_| "plugin requested abort".to_string());
                    return HookResult::Abort { reason };
                } else if result.to_lowercase() == "skip" {
                    return HookResult::Skip;
                }
            }
            HookResult::Continue
        }
        _ => HookResult::Continue,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::time::Duration;
    use tempfile::NamedTempFile;

    fn create_test_track() -> Track {
        Track::new(
            std::path::PathBuf::from("/music/test.mp3"),
            "Test Song".to_string(),
            "Test Artist".to_string(),
            Duration::from_secs(180),
        )
    }

    fn create_plugin_file(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "{content}").unwrap();
        file
    }

    #[test]
    fn test_runtime_creation() {
        let runtime = LuaRuntime::new().unwrap();
        assert!(runtime.plugins().is_empty());
    }

    #[test]
    fn test_load_simple_plugin() {
        let mut runtime = LuaRuntime::new().unwrap();

        let plugin_file = create_plugin_file(
            r#"
            local plugin = {
                name = "test_plugin",
                version = "1.0.0",
                description = "A test plugin",
            }
            return plugin
        "#,
        );

        let plugin = runtime.load_plugin(plugin_file.path()).unwrap();
        assert_eq!(plugin.name, "test_plugin");
        assert_eq!(plugin.version, "1.0.0");
    }

    #[test]
    fn test_on_import_hook() {
        let mut runtime = LuaRuntime::new().unwrap();

        let plugin_file = create_plugin_file(
            r#"
            local plugin = {
                name = "import_test",
                version = "1.0.0",
                description = "Test import hook",
            }

            function plugin.on_import(track)
                track.title = "Modified Title"
                track.artist = "Modified Artist"
                return "continue"
            end

            return plugin
        "#,
        );

        runtime.load_plugin(plugin_file.path()).unwrap();

        let mut track = create_test_track();
        let result = runtime.run_on_import(&mut track).unwrap();

        assert_eq!(result, HookResult::Continue);
        assert_eq!(track.title, "Modified Title");
        assert_eq!(track.artist, "Modified Artist");
    }

    #[test]
    fn test_on_import_hook_skip() {
        let mut runtime = LuaRuntime::new().unwrap();

        let plugin_file = create_plugin_file(
            r#"
            local plugin = {
                name = "skip_test",
                version = "1.0.0",
                description = "Test skip result",
            }

            function plugin.on_import(track)
                return "skip"
            end

            return plugin
        "#,
        );

        runtime.load_plugin(plugin_file.path()).unwrap();

        let mut track = create_test_track();
        let result = runtime.run_on_import(&mut track).unwrap();

        assert_eq!(result, HookResult::Skip);
    }

    #[test]
    fn test_on_import_hook_abort() {
        let mut runtime = LuaRuntime::new().unwrap();

        let plugin_file = create_plugin_file(
            r#"
            local plugin = {
                name = "abort_test",
                version = "1.0.0",
                description = "Test abort result",
            }

            function plugin.on_import(track)
                return { result = "abort", reason = "Test abort reason" }
            end

            return plugin
        "#,
        );

        runtime.load_plugin(plugin_file.path()).unwrap();

        let mut track = create_test_track();
        let result = runtime.run_on_import(&mut track).unwrap();

        assert!(matches!(result, HookResult::Abort { reason } if reason == "Test abort reason"));
    }

    #[test]
    fn test_multiple_hooks() {
        let mut runtime = LuaRuntime::new().unwrap();

        let plugin1 = create_plugin_file(
            r#"
            local plugin = {
                name = "plugin1",
                version = "1.0.0",
                description = "First plugin",
            }

            function plugin.on_import(track)
                track.title = track.title .. " [1]"
                return "continue"
            end

            return plugin
        "#,
        );

        let plugin2 = create_plugin_file(
            r#"
            local plugin = {
                name = "plugin2",
                version = "1.0.0",
                description = "Second plugin",
            }

            function plugin.on_import(track)
                track.title = track.title .. " [2]"
                return "continue"
            end

            return plugin
        "#,
        );

        runtime.load_plugin(plugin1.path()).unwrap();
        runtime.load_plugin(plugin2.path()).unwrap();

        let mut track = create_test_track();
        runtime.run_on_import(&mut track).unwrap();

        // Both plugins should have modified the title
        assert!(track.title.contains("[1]"));
        assert!(track.title.contains("[2]"));
    }

    #[test]
    fn test_exec_lua_code() {
        let runtime = LuaRuntime::new().unwrap();

        runtime.exec("apollo.info('Hello from Lua!')").unwrap();

        let result: i32 = runtime.eval("return 1 + 2").unwrap();
        assert_eq!(result, 3);
    }

    #[test]
    fn test_no_hooks_returns_continue() {
        let runtime = LuaRuntime::new().unwrap();

        let mut track = create_test_track();
        let result = runtime.run_on_import(&mut track).unwrap();

        assert_eq!(result, HookResult::Continue);
    }

    #[test]
    fn test_on_album_import_hook() {
        let mut runtime = LuaRuntime::new().unwrap();

        let plugin_file = create_plugin_file(
            r#"
            local plugin = {
                name = "album_test",
                version = "1.0.0",
                description = "Test album import hook",
            }

            function plugin.on_album_import(album)
                album.title = "Modified Album"
                album.year = 2024
                return "continue"
            end

            return plugin
        "#,
        );

        runtime.load_plugin(plugin_file.path()).unwrap();

        let mut album = Album::new("Original Album".to_string(), "Test Artist".to_string());
        let result = runtime.run_on_album_import(&mut album).unwrap();

        assert_eq!(result, HookResult::Continue);
        assert_eq!(album.title, "Modified Album");
        assert_eq!(album.year, Some(2024));
    }

    #[test]
    fn test_parse_hook_result() {
        assert_eq!(parse_hook_result(&Value::Nil), HookResult::Continue);

        let lua = Lua::new();
        let continue_str = lua.create_string("continue").unwrap();
        assert_eq!(
            parse_hook_result(&Value::String(continue_str)),
            HookResult::Continue
        );

        let skip_str = lua.create_string("skip").unwrap();
        assert_eq!(
            parse_hook_result(&Value::String(skip_str)),
            HookResult::Skip
        );

        let abort_str = lua.create_string("abort").unwrap();
        assert!(matches!(
            parse_hook_result(&Value::String(abort_str)),
            HookResult::Abort { .. }
        ));
    }
}
