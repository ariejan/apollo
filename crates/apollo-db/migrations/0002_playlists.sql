-- Apollo Music Library Schema
-- Migration: 0002_playlists
-- Description: Add playlists and playlist_tracks tables

-- Playlists table
CREATE TABLE IF NOT EXISTS playlists (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    kind TEXT NOT NULL DEFAULT 'static',  -- 'static' or 'smart'
    query TEXT,  -- JSON-serialized Query for smart playlists
    sort TEXT NOT NULL DEFAULT 'artist',
    max_tracks INTEGER,
    max_duration_secs INTEGER,
    created_at TEXT NOT NULL,  -- ISO8601 timestamp
    modified_at TEXT NOT NULL
);

-- Create indexes for playlist queries
CREATE INDEX IF NOT EXISTS idx_playlists_name ON playlists(name);
CREATE INDEX IF NOT EXISTS idx_playlists_kind ON playlists(kind);

-- Playlist tracks junction table (for static playlists)
-- Stores the ordered list of tracks in a playlist
CREATE TABLE IF NOT EXISTS playlist_tracks (
    playlist_id TEXT NOT NULL REFERENCES playlists(id) ON DELETE CASCADE,
    track_id TEXT NOT NULL REFERENCES tracks(id) ON DELETE CASCADE,
    position INTEGER NOT NULL,  -- Order within the playlist
    added_at TEXT NOT NULL,  -- ISO8601 timestamp
    PRIMARY KEY (playlist_id, track_id)
);

-- Create index for fetching tracks in a playlist
CREATE INDEX IF NOT EXISTS idx_playlist_tracks_playlist ON playlist_tracks(playlist_id, position);
