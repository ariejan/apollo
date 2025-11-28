-- Apollo Music Library Schema
-- Migration: 0001_initial_schema
-- Description: Create initial tables for tracks and albums

-- Albums table
CREATE TABLE IF NOT EXISTS albums (
    id TEXT PRIMARY KEY NOT NULL,
    title TEXT NOT NULL,
    artist TEXT NOT NULL,
    year INTEGER,
    genres TEXT NOT NULL DEFAULT '[]',  -- JSON array of genre strings
    track_count INTEGER NOT NULL DEFAULT 0,
    disc_count INTEGER NOT NULL DEFAULT 1,
    musicbrainz_id TEXT,
    added_at TEXT NOT NULL,  -- ISO8601 timestamp
    modified_at TEXT NOT NULL
);

-- Create indexes for common album queries
CREATE INDEX IF NOT EXISTS idx_albums_artist ON albums(artist);
CREATE INDEX IF NOT EXISTS idx_albums_title ON albums(title);
CREATE INDEX IF NOT EXISTS idx_albums_year ON albums(year);
CREATE INDEX IF NOT EXISTS idx_albums_musicbrainz_id ON albums(musicbrainz_id);

-- Tracks table
CREATE TABLE IF NOT EXISTS tracks (
    id TEXT PRIMARY KEY NOT NULL,
    path TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    artist TEXT NOT NULL,
    album_artist TEXT,
    album_id TEXT REFERENCES albums(id) ON DELETE SET NULL,
    album_title TEXT,
    track_number INTEGER,
    track_total INTEGER,
    disc_number INTEGER,
    disc_total INTEGER,
    year INTEGER,
    genres TEXT NOT NULL DEFAULT '[]',  -- JSON array of genre strings
    duration_ms INTEGER NOT NULL,
    bitrate INTEGER,
    sample_rate INTEGER,
    channels INTEGER,
    format TEXT NOT NULL,
    musicbrainz_id TEXT,
    acoustid TEXT,
    added_at TEXT NOT NULL,  -- ISO8601 timestamp
    modified_at TEXT NOT NULL,
    file_hash TEXT NOT NULL
);

-- Create indexes for common track queries
CREATE INDEX IF NOT EXISTS idx_tracks_artist ON tracks(artist);
CREATE INDEX IF NOT EXISTS idx_tracks_album_artist ON tracks(album_artist);
CREATE INDEX IF NOT EXISTS idx_tracks_album_id ON tracks(album_id);
CREATE INDEX IF NOT EXISTS idx_tracks_album_title ON tracks(album_title);
CREATE INDEX IF NOT EXISTS idx_tracks_title ON tracks(title);
CREATE INDEX IF NOT EXISTS idx_tracks_year ON tracks(year);
CREATE INDEX IF NOT EXISTS idx_tracks_format ON tracks(format);
CREATE INDEX IF NOT EXISTS idx_tracks_musicbrainz_id ON tracks(musicbrainz_id);
CREATE INDEX IF NOT EXISTS idx_tracks_file_hash ON tracks(file_hash);

-- Artists view (derived from tracks and albums)
-- This is a virtual table that aggregates artist information
CREATE VIEW IF NOT EXISTS artists AS
SELECT DISTINCT
    artist as name,
    NULL as sort_name,
    NULL as musicbrainz_id
FROM tracks
UNION
SELECT DISTINCT
    album_artist as name,
    NULL as sort_name,
    NULL as musicbrainz_id
FROM tracks
WHERE album_artist IS NOT NULL
UNION
SELECT DISTINCT
    artist as name,
    NULL as sort_name,
    NULL as musicbrainz_id
FROM albums;

-- Full-text search virtual table for tracks
CREATE VIRTUAL TABLE IF NOT EXISTS tracks_fts USING fts5(
    title,
    artist,
    album_artist,
    album_title,
    content='tracks',
    content_rowid='rowid'
);

-- Triggers to keep FTS index in sync
CREATE TRIGGER IF NOT EXISTS tracks_ai AFTER INSERT ON tracks BEGIN
    INSERT INTO tracks_fts(rowid, title, artist, album_artist, album_title)
    VALUES (new.rowid, new.title, new.artist, new.album_artist, new.album_title);
END;

CREATE TRIGGER IF NOT EXISTS tracks_ad AFTER DELETE ON tracks BEGIN
    INSERT INTO tracks_fts(tracks_fts, rowid, title, artist, album_artist, album_title)
    VALUES ('delete', old.rowid, old.title, old.artist, old.album_artist, old.album_title);
END;

CREATE TRIGGER IF NOT EXISTS tracks_au AFTER UPDATE ON tracks BEGIN
    INSERT INTO tracks_fts(tracks_fts, rowid, title, artist, album_artist, album_title)
    VALUES ('delete', old.rowid, old.title, old.artist, old.album_artist, old.album_title);
    INSERT INTO tracks_fts(rowid, title, artist, album_artist, album_title)
    VALUES (new.rowid, new.title, new.artist, new.album_artist, new.album_title);
END;
