// TypeScript types matching the Apollo REST API

export interface Track {
  id: string;
  title: string;
  artist: string;
  album: string;
  album_id: string | null;
  track_number: number | null;
  disc_number: number | null;
  duration_ms: number | null;
  year: number | null;
  genres: string[];
  file_path: string;
  file_hash: string | null;
  format: string;
  bitrate: number | null;
  sample_rate: number | null;
  channels: number | null;
  musicbrainz_id: string | null;
  acoustid: string | null;
  added_at: string;
  updated_at: string;
}

export interface Album {
  id: string;
  title: string;
  artist: string;
  year: number | null;
  genres: string[];
  total_tracks: number | null;
  total_discs: number | null;
  musicbrainz_id: string | null;
  cover_art_path: string | null;
  added_at: string;
  updated_at: string;
}

export interface Playlist {
  id: string;
  name: string;
  description: string | null;
  playlist_type: 'static' | 'smart';
  query: string | null;
  sort_order: string | null;
  max_tracks: number | null;
  track_count: number;
  created_at: string;
  updated_at: string;
}

export interface Stats {
  track_count: number;
  album_count: number;
  playlist_count: number;
}

export interface Health {
  status: string;
  version: string;
}

export interface PaginatedResponse<T> {
  items: T[];
  total: number;
  limit: number;
  offset: number;
}

export interface ErrorResponse {
  error_type: string;
  message: string;
}

export interface CreatePlaylistRequest {
  name: string;
  description?: string;
  playlist_type: 'static' | 'smart';
  query?: string;
  sort_order?: string;
  max_tracks?: number;
}

export interface UpdatePlaylistRequest {
  name?: string;
  description?: string;
  query?: string;
  sort_order?: string;
  max_tracks?: number;
}

export interface AddTracksRequest {
  track_ids: string[];
}
