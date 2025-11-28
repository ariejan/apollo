// Apollo API client

import type {
  Track,
  Album,
  Playlist,
  Stats,
  Health,
  PaginatedResponse,
  CreatePlaylistRequest,
  UpdatePlaylistRequest,
  AddTracksRequest,
} from './types';

const API_BASE = '/api';

async function fetchJson<T>(url: string, options?: RequestInit): Promise<T> {
  const response = await fetch(url, {
    headers: {
      'Content-Type': 'application/json',
      ...options?.headers,
    },
    ...options,
  });

  if (!response.ok) {
    const error = await response.json().catch(() => ({ message: response.statusText }));
    throw new Error(error.message || `HTTP ${response.status}`);
  }

  return response.json();
}

// Health check
export async function getHealth(): Promise<Health> {
  return fetchJson<Health>('/health');
}

// Stats
export async function getStats(): Promise<Stats> {
  return fetchJson<Stats>(`${API_BASE}/stats`);
}

// Tracks
export async function getTracks(limit = 50, offset = 0): Promise<PaginatedResponse<Track>> {
  return fetchJson<PaginatedResponse<Track>>(`${API_BASE}/tracks?limit=${limit}&offset=${offset}`);
}

export async function getTrack(id: string): Promise<Track> {
  return fetchJson<Track>(`${API_BASE}/tracks/${id}`);
}

// Albums
export async function getAlbums(limit = 50, offset = 0): Promise<PaginatedResponse<Album>> {
  return fetchJson<PaginatedResponse<Album>>(`${API_BASE}/albums?limit=${limit}&offset=${offset}`);
}

export async function getAlbum(id: string): Promise<Album> {
  return fetchJson<Album>(`${API_BASE}/albums/${id}`);
}

export async function getAlbumTracks(id: string): Promise<Track[]> {
  return fetchJson<Track[]>(`${API_BASE}/albums/${id}/tracks`);
}

// Search
export async function searchTracks(query: string, limit = 50, offset = 0): Promise<PaginatedResponse<Track>> {
  const encodedQuery = encodeURIComponent(query);
  return fetchJson<PaginatedResponse<Track>>(`${API_BASE}/search?q=${encodedQuery}&limit=${limit}&offset=${offset}`);
}

// Playlists
export async function getPlaylists(): Promise<Playlist[]> {
  return fetchJson<Playlist[]>(`${API_BASE}/playlists`);
}

export async function getPlaylist(id: string): Promise<Playlist> {
  return fetchJson<Playlist>(`${API_BASE}/playlists/${id}`);
}

export async function getPlaylistTracks(id: string): Promise<Track[]> {
  return fetchJson<Track[]>(`${API_BASE}/playlists/${id}/tracks`);
}

export async function createPlaylist(request: CreatePlaylistRequest): Promise<Playlist> {
  return fetchJson<Playlist>(`${API_BASE}/playlists`, {
    method: 'POST',
    body: JSON.stringify(request),
  });
}

export async function updatePlaylist(id: string, request: UpdatePlaylistRequest): Promise<Playlist> {
  return fetchJson<Playlist>(`${API_BASE}/playlists/${id}`, {
    method: 'PATCH',
    body: JSON.stringify(request),
  });
}

export async function deletePlaylist(id: string): Promise<void> {
  await fetch(`${API_BASE}/playlists/${id}`, { method: 'DELETE' });
}

export async function addTracksToPlaylist(id: string, trackIds: string[]): Promise<void> {
  await fetchJson<void>(`${API_BASE}/playlists/${id}/tracks`, {
    method: 'POST',
    body: JSON.stringify({ track_ids: trackIds } as AddTracksRequest),
  });
}

export async function removeTracksFromPlaylist(id: string, trackIds: string[]): Promise<void> {
  await fetchJson<void>(`${API_BASE}/playlists/${id}/tracks`, {
    method: 'DELETE',
    body: JSON.stringify({ track_ids: trackIds } as AddTracksRequest),
  });
}

// Utility functions
export function formatDuration(ms: number | null): string {
  if (ms === null) return '--:--';
  const totalSeconds = Math.floor(ms / 1000);
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;
  return `${minutes}:${seconds.toString().padStart(2, '0')}`;
}

export function formatDate(isoDate: string): string {
  return new Date(isoDate).toLocaleDateString();
}
