//! Lua bindings for Apollo core types.
//!
//! This module provides Lua userdata implementations for [`Track`] and [`Album`],
//! allowing them to be created, accessed, and modified from Lua scripts.

// Allow these clippy lints for the bindings module since they're mostly noise
// for the types of closures used in mlua userdata implementations.
#![allow(clippy::significant_drop_tightening)]
#![allow(clippy::missing_const_for_fn)]

use apollo_core::{Album, Track};
use mlua::{FromLua, IntoLua, Lua, MetaMethod, Result, UserData, UserDataMethods, Value};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::Duration;

/// A wrapper around [`Track`] that can be shared with Lua.
///
/// This wrapper uses `Arc<RwLock<Track>>` to allow both Rust and Lua
/// to read and modify the track data safely.
#[derive(Clone)]
pub struct LuaTrack(pub Arc<RwLock<Track>>);

impl LuaTrack {
    /// Create a new `LuaTrack` from a `Track`.
    #[must_use]
    pub fn new(track: Track) -> Self {
        Self(Arc::new(RwLock::new(track)))
    }

    /// Create a new `LuaTrack` from an existing `Arc<RwLock<Track>>`.
    #[must_use]
    #[allow(dead_code)]
    pub fn from_shared(track: Arc<RwLock<Track>>) -> Self {
        Self(track)
    }

    /// Get a clone of the inner track.
    ///
    /// # Panics
    ///
    /// Panics if the lock is poisoned.
    #[must_use]
    pub fn get(&self) -> Track {
        self.0.read().expect("lock poisoned").clone()
    }

    /// Replace the inner track.
    ///
    /// # Panics
    ///
    /// Panics if the lock is poisoned.
    #[allow(dead_code)]
    pub fn set(&self, track: Track) {
        *self.0.write().expect("lock poisoned") = track;
    }
}

impl UserData for LuaTrack {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        // Read-only properties
        methods.add_meta_method(MetaMethod::Index, |lua, this, key: String| {
            let track = this
                .0
                .read()
                .map_err(|_| mlua::Error::runtime("lock poisoned"))?;
            match key.as_str() {
                "id" => track.id.to_string().into_lua(lua),
                "path" => track.path.to_string_lossy().to_string().into_lua(lua),
                "title" => track.title.clone().into_lua(lua),
                "artist" => track.artist.clone().into_lua(lua),
                "album_artist" => track.album_artist.clone().into_lua(lua),
                "album_title" => track.album_title.clone().into_lua(lua),
                "track_number" => track.track_number.into_lua(lua),
                "track_total" => track.track_total.into_lua(lua),
                "disc_number" => track.disc_number.into_lua(lua),
                "disc_total" => track.disc_total.into_lua(lua),
                "year" => track.year.into_lua(lua),
                "genres" => track.genres.clone().into_lua(lua),
                "duration" => (track.duration.as_secs_f64()).into_lua(lua),
                #[allow(clippy::cast_possible_truncation)] // 584 million years before truncation
                "duration_ms" => (track.duration.as_millis() as u64).into_lua(lua),
                "bitrate" => track.bitrate.into_lua(lua),
                "sample_rate" => track.sample_rate.into_lua(lua),
                "channels" => track.channels.into_lua(lua),
                "format" => track.format.to_string().into_lua(lua),
                "musicbrainz_id" => track.musicbrainz_id.clone().into_lua(lua),
                "acoustid" => track.acoustid.clone().into_lua(lua),
                "file_hash" => track.file_hash.clone().into_lua(lua),
                _ => Ok(Value::Nil),
            }
        });

        // Mutable properties
        methods.add_meta_method_mut(
            MetaMethod::NewIndex,
            |lua, this, (key, value): (String, Value)| {
                let mut track = this
                    .0
                    .write()
                    .map_err(|_| mlua::Error::runtime("lock poisoned"))?;
                match key.as_str() {
                    "title" => {
                        track.title = String::from_lua(value, lua)?;
                    }
                    "artist" => {
                        track.artist = String::from_lua(value, lua)?;
                    }
                    "album_artist" => {
                        track.album_artist = Option::<String>::from_lua(value, lua)?;
                    }
                    "album_title" => {
                        track.album_title = Option::<String>::from_lua(value, lua)?;
                    }
                    "track_number" => {
                        track.track_number = Option::<u32>::from_lua(value, lua)?;
                    }
                    "track_total" => {
                        track.track_total = Option::<u32>::from_lua(value, lua)?;
                    }
                    "disc_number" => {
                        track.disc_number = Option::<u32>::from_lua(value, lua)?;
                    }
                    "disc_total" => {
                        track.disc_total = Option::<u32>::from_lua(value, lua)?;
                    }
                    "year" => {
                        track.year = Option::<i32>::from_lua(value, lua)?;
                    }
                    "genres" => {
                        track.genres = Vec::<String>::from_lua(value, lua)?;
                    }
                    "musicbrainz_id" => {
                        track.musicbrainz_id = Option::<String>::from_lua(value, lua)?;
                    }
                    "acoustid" => {
                        track.acoustid = Option::<String>::from_lua(value, lua)?;
                    }
                    _ => {
                        return Err(mlua::Error::runtime(format!(
                            "cannot set property '{key}' (read-only or unknown)"
                        )));
                    }
                }
                Ok(())
            },
        );

        // String representation
        methods.add_meta_method(MetaMethod::ToString, |_, this, ()| {
            let track = this
                .0
                .read()
                .map_err(|_| mlua::Error::runtime("lock poisoned"))?;
            Ok(format!("Track({} - {})", track.artist, track.title))
        });
    }
}

/// A wrapper around [`Album`] that can be shared with Lua.
#[derive(Clone)]
pub struct LuaAlbum(pub Arc<RwLock<Album>>);

impl LuaAlbum {
    /// Create a new `LuaAlbum` from an `Album`.
    #[must_use]
    pub fn new(album: Album) -> Self {
        Self(Arc::new(RwLock::new(album)))
    }

    /// Create a new `LuaAlbum` from an existing `Arc<RwLock<Album>>`.
    #[must_use]
    #[allow(dead_code)]
    pub fn from_shared(album: Arc<RwLock<Album>>) -> Self {
        Self(album)
    }

    /// Get a clone of the inner album.
    ///
    /// # Panics
    ///
    /// Panics if the lock is poisoned.
    #[must_use]
    pub fn get(&self) -> Album {
        self.0.read().expect("lock poisoned").clone()
    }

    /// Replace the inner album.
    ///
    /// # Panics
    ///
    /// Panics if the lock is poisoned.
    #[allow(dead_code)]
    pub fn set(&self, album: Album) {
        *self.0.write().expect("lock poisoned") = album;
    }
}

impl UserData for LuaAlbum {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        // Read-only properties
        methods.add_meta_method(MetaMethod::Index, |lua, this, key: String| {
            let album = this
                .0
                .read()
                .map_err(|_| mlua::Error::runtime("lock poisoned"))?;
            match key.as_str() {
                "id" => album.id.to_string().into_lua(lua),
                "title" => album.title.clone().into_lua(lua),
                "artist" => album.artist.clone().into_lua(lua),
                "year" => album.year.into_lua(lua),
                "genres" => album.genres.clone().into_lua(lua),
                "track_count" => album.track_count.into_lua(lua),
                "disc_count" => album.disc_count.into_lua(lua),
                "musicbrainz_id" => album.musicbrainz_id.clone().into_lua(lua),
                _ => Ok(Value::Nil),
            }
        });

        // Mutable properties
        methods.add_meta_method_mut(
            MetaMethod::NewIndex,
            |lua, this, (key, value): (String, Value)| {
                let mut album = this
                    .0
                    .write()
                    .map_err(|_| mlua::Error::runtime("lock poisoned"))?;
                match key.as_str() {
                    "title" => {
                        album.title = String::from_lua(value, lua)?;
                    }
                    "artist" => {
                        album.artist = String::from_lua(value, lua)?;
                    }
                    "year" => {
                        album.year = Option::<i32>::from_lua(value, lua)?;
                    }
                    "genres" => {
                        album.genres = Vec::<String>::from_lua(value, lua)?;
                    }
                    "track_count" => {
                        album.track_count = u32::from_lua(value, lua)?;
                    }
                    "disc_count" => {
                        album.disc_count = u32::from_lua(value, lua)?;
                    }
                    "musicbrainz_id" => {
                        album.musicbrainz_id = Option::<String>::from_lua(value, lua)?;
                    }
                    _ => {
                        return Err(mlua::Error::runtime(format!(
                            "cannot set property '{key}' (read-only or unknown)"
                        )));
                    }
                }
                Ok(())
            },
        );

        // String representation
        methods.add_meta_method(MetaMethod::ToString, |_, this, ()| {
            let album = this
                .0
                .read()
                .map_err(|_| mlua::Error::runtime("lock poisoned"))?;
            Ok(format!("Album({} - {})", album.artist, album.title))
        });
    }
}

/// Register the Apollo module with the Lua runtime.
///
/// This creates the `apollo` global table with factory functions for creating
/// tracks and albums.
///
/// # Errors
///
/// Returns an error if registration fails.
pub fn register_apollo_module(lua: &Lua) -> Result<()> {
    let apollo = lua.create_table()?;

    // apollo.new_track(path, title, artist, duration_secs)
    apollo.set(
        "new_track",
        lua.create_function(
            |_, (path, title, artist, duration_secs): (String, String, String, f64)| {
                let track = Track::new(
                    PathBuf::from(path),
                    title,
                    artist,
                    Duration::from_secs_f64(duration_secs),
                );
                Ok(LuaTrack::new(track))
            },
        )?,
    )?;

    // apollo.new_album(title, artist)
    apollo.set(
        "new_album",
        lua.create_function(|_, (title, artist): (String, String)| {
            let album = Album::new(title, artist);
            Ok(LuaAlbum::new(album))
        })?,
    )?;

    // apollo.log(level, message)
    apollo.set(
        "log",
        lua.create_function(|_, (level, message): (String, String)| {
            match level.to_lowercase().as_str() {
                "error" => tracing::error!("[lua] {}", message),
                "warn" => tracing::warn!("[lua] {}", message),
                "info" => tracing::info!("[lua] {}", message),
                "debug" => tracing::debug!("[lua] {}", message),
                "trace" => tracing::trace!("[lua] {}", message),
                _ => tracing::info!("[lua] {}", message),
            }
            Ok(())
        })?,
    )?;

    // Convenience logging functions
    apollo.set(
        "info",
        lua.create_function(|_, message: String| {
            tracing::info!("[lua] {}", message);
            Ok(())
        })?,
    )?;

    apollo.set(
        "debug",
        lua.create_function(|_, message: String| {
            tracing::debug!("[lua] {}", message);
            Ok(())
        })?,
    )?;

    apollo.set(
        "warn",
        lua.create_function(|_, message: String| {
            tracing::warn!("[lua] {}", message);
            Ok(())
        })?,
    )?;

    apollo.set(
        "error",
        lua.create_function(|_, message: String| {
            tracing::error!("[lua] {}", message);
            Ok(())
        })?,
    )?;

    // apollo.version
    apollo.set("version", env!("CARGO_PKG_VERSION"))?;

    lua.globals().set("apollo", apollo)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lua_track_read_properties() {
        let lua = Lua::new();
        register_apollo_module(&lua).unwrap();

        let track = Track::new(
            PathBuf::from("/music/test.mp3"),
            "Test Song".to_string(),
            "Test Artist".to_string(),
            Duration::from_secs(180),
        );

        lua.scope(|scope| {
            let lua_track = scope.create_userdata(LuaTrack::new(track)).unwrap();
            lua.globals().set("track", lua_track).unwrap();

            let title: String = lua.load("return track.title").eval().unwrap();
            assert_eq!(title, "Test Song");

            let artist: String = lua.load("return track.artist").eval().unwrap();
            assert_eq!(artist, "Test Artist");

            let duration: f64 = lua.load("return track.duration").eval().unwrap();
            assert!((duration - 180.0).abs() < 0.001);

            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn test_lua_track_write_properties() {
        let lua = Lua::new();
        register_apollo_module(&lua).unwrap();

        let track = Track::new(
            PathBuf::from("/music/test.mp3"),
            "Original Title".to_string(),
            "Original Artist".to_string(),
            Duration::from_secs(180),
        );

        let lua_track = LuaTrack::new(track);
        lua.globals().set("track", lua_track.clone()).unwrap();

        lua.load(
            r#"
            track.title = "New Title"
            track.artist = "New Artist"
            track.year = 2024
            track.genres = {"Rock", "Alternative"}
        "#,
        )
        .exec()
        .unwrap();

        let modified = lua_track.get();
        assert_eq!(modified.title, "New Title");
        assert_eq!(modified.artist, "New Artist");
        assert_eq!(modified.year, Some(2024));
        assert_eq!(modified.genres, vec!["Rock", "Alternative"]);
    }

    #[test]
    fn test_lua_track_tostring() {
        let lua = Lua::new();
        register_apollo_module(&lua).unwrap();

        let track = Track::new(
            PathBuf::from("/music/test.mp3"),
            "My Song".to_string(),
            "My Artist".to_string(),
            Duration::from_secs(180),
        );

        lua.globals().set("track", LuaTrack::new(track)).unwrap();

        let result: String = lua.load("return tostring(track)").eval().unwrap();
        assert_eq!(result, "Track(My Artist - My Song)");
    }

    #[test]
    fn test_lua_album_read_properties() {
        let lua = Lua::new();
        register_apollo_module(&lua).unwrap();

        let mut album = Album::new("Test Album".to_string(), "Test Artist".to_string());
        album.year = Some(2024);
        album.track_count = 12;

        lua.globals().set("album", LuaAlbum::new(album)).unwrap();

        let title: String = lua.load("return album.title").eval().unwrap();
        assert_eq!(title, "Test Album");

        let year: i32 = lua.load("return album.year").eval().unwrap();
        assert_eq!(year, 2024);

        let track_count: u32 = lua.load("return album.track_count").eval().unwrap();
        assert_eq!(track_count, 12);
    }

    #[test]
    fn test_lua_album_write_properties() {
        let lua = Lua::new();
        register_apollo_module(&lua).unwrap();

        let album = Album::new("Original Album".to_string(), "Original Artist".to_string());

        let lua_album = LuaAlbum::new(album);
        lua.globals().set("album", lua_album.clone()).unwrap();

        lua.load(
            r#"
            album.title = "New Album Title"
            album.artist = "New Album Artist"
            album.year = 2023
        "#,
        )
        .exec()
        .unwrap();

        let modified = lua_album.get();
        assert_eq!(modified.title, "New Album Title");
        assert_eq!(modified.artist, "New Album Artist");
        assert_eq!(modified.year, Some(2023));
    }

    #[test]
    fn test_apollo_module_new_track() {
        let lua = Lua::new();
        register_apollo_module(&lua).unwrap();

        lua.load(
            r#"
            local track = apollo.new_track("/music/new.mp3", "New Song", "New Artist", 240.5)
            assert(track.title == "New Song")
            assert(track.artist == "New Artist")
            assert(track.path == "/music/new.mp3")
            -- Duration is in seconds
            assert(math.abs(track.duration - 240.5) < 0.001)
        "#,
        )
        .exec()
        .unwrap();
    }

    #[test]
    fn test_apollo_module_new_album() {
        let lua = Lua::new();
        register_apollo_module(&lua).unwrap();

        lua.load(
            r#"
            local album = apollo.new_album("New Album", "New Artist")
            assert(album.title == "New Album")
            assert(album.artist == "New Artist")
        "#,
        )
        .exec()
        .unwrap();
    }

    #[test]
    fn test_apollo_module_version() {
        let lua = Lua::new();
        register_apollo_module(&lua).unwrap();

        let version: String = lua.load("return apollo.version").eval().unwrap();
        assert_eq!(version, env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn test_read_only_property_error() {
        let lua = Lua::new();
        register_apollo_module(&lua).unwrap();

        let track = Track::new(
            PathBuf::from("/music/test.mp3"),
            "Test Song".to_string(),
            "Test Artist".to_string(),
            Duration::from_secs(180),
        );

        lua.globals().set("track", LuaTrack::new(track)).unwrap();

        // path is read-only
        let result = lua.load("track.path = '/other/path.mp3'").exec();
        assert!(result.is_err());
    }
}
