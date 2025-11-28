<script lang="ts">
  import Header from '../lib/components/Header.svelte';
  import Loading from '../lib/components/Loading.svelte';
  import EmptyState from '../lib/components/EmptyState.svelte';
  import TrackRow from '../lib/components/TrackRow.svelte';
  import { link } from 'svelte-spa-router';
  import { getPlaylist, getPlaylistTracks, formatDuration, formatDate } from '../lib/api/client';
  import type { Playlist, Track } from '../lib/api/types';

  interface Props {
    params: { id: string };
  }

  let { params }: Props = $props();

  let playlist: Playlist | null = $state(null);
  let tracks: Track[] = $state([]);
  let loading = $state(true);
  let error: string | null = $state(null);

  $effect(() => {
    loadPlaylist(params.id);
  });

  async function loadPlaylist(id: string) {
    try {
      loading = true;
      error = null;
      const [playlistData, tracksData] = await Promise.all([
        getPlaylist(id),
        getPlaylistTracks(id),
      ]);
      playlist = playlistData;
      tracks = tracksData;
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to load playlist';
    } finally {
      loading = false;
    }
  }

  const totalDuration = $derived(
    tracks.reduce((sum, t) => sum + (t.duration_ms ?? 0), 0)
  );
</script>

<Header title={playlist?.name ?? 'Playlist'} />

<div class="content">
  {#if loading}
    <Loading />
  {:else if error}
    <div class="error">
      <p>Error: {error}</p>
      <a href="/playlists" use:link class="back-link">Back to Playlists</a>
    </div>
  {:else if playlist}
    <div class="playlist-header">
      <div class="playlist-icon">
        {#if playlist.playlist_type === 'smart'}
          <svg viewBox="0 0 24 24" fill="currentColor"><path d="M14 2H6c-1.1 0-2 .9-2 2v16c0 1.1.9 2 2 2h12c1.1 0 2-.9 2-2V8l-6-6zm4 18H6V4h7v5h5v11zm-3-4h-2v2h-2v-2H9v-2h2v-2h2v2h2v2z"/></svg>
        {:else}
          <svg viewBox="0 0 24 24" fill="currentColor"><path d="M15 6H3v2h12V6zm0 4H3v2h12v-2zM3 16h8v-2H3v2zM17 6v8.18c-.31-.11-.65-.18-1-.18-1.66 0-3 1.34-3 3s1.34 3 3 3 3-1.34 3-3V8h3V6h-5z"/></svg>
        {/if}
      </div>
      <div class="playlist-info">
        <span class="playlist-type">
          {playlist.playlist_type === 'smart' ? 'Smart Playlist' : 'Playlist'}
        </span>
        <h1>{playlist.name}</h1>
        {#if playlist.description}
          <p class="playlist-description">{playlist.description}</p>
        {/if}
        <div class="playlist-meta">
          <span>{tracks.length} tracks</span>
          <span class="separator">-</span>
          <span>{formatDuration(totalDuration)}</span>
          <span class="separator">-</span>
          <span>Created {formatDate(playlist.created_at)}</span>
        </div>
        {#if playlist.playlist_type === 'smart' && playlist.query}
          <div class="playlist-query">
            <span class="query-label">Query:</span>
            <code>{playlist.query}</code>
          </div>
        {/if}
      </div>
    </div>

    {#if tracks.length === 0}
      <EmptyState message="No tracks in this playlist" icon="playlist" />
    {:else}
      <div class="tracks-section">
        <div class="tracks-header">
          <div class="header-number">#</div>
          <div class="header-title">Title / Artist</div>
          <div class="header-album">Album</div>
          <div class="header-duration">Duration</div>
          <div class="header-format">Format</div>
        </div>

        <div class="tracks-list">
          {#each tracks as track}
            <TrackRow {track} />
          {/each}
        </div>
      </div>
    {/if}
  {/if}
</div>

<style>
  .content {
    padding: 2rem;
  }

  .playlist-header {
    display: flex;
    gap: 2rem;
    margin-bottom: 2rem;
  }

  .playlist-icon {
    width: 180px;
    height: 180px;
    flex-shrink: 0;
    background: #2a2a4a;
    border-radius: 8px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: #667eea;
  }

  .playlist-icon svg {
    width: 80px;
    height: 80px;
  }

  .playlist-info {
    display: flex;
    flex-direction: column;
    justify-content: flex-end;
  }

  .playlist-type {
    font-size: 0.75rem;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: #808090;
    margin-bottom: 0.5rem;
  }

  .playlist-info h1 {
    margin: 0 0 0.5rem;
    font-size: 2rem;
    font-weight: 700;
    color: #fff;
  }

  .playlist-description {
    margin: 0 0 0.75rem;
    font-size: 1rem;
    color: #a0a0c0;
  }

  .playlist-meta {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    color: #808090;
    font-size: 0.875rem;
    margin-bottom: 0.75rem;
  }

  .separator {
    color: #4a4a6a;
  }

  .playlist-query {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .query-label {
    color: #606080;
    font-size: 0.875rem;
  }

  .playlist-query code {
    background: #1a1a2e;
    padding: 0.25rem 0.75rem;
    border-radius: 4px;
    font-family: 'Fira Code', monospace;
    font-size: 0.875rem;
    color: #c0c0e0;
  }

  .tracks-section {
    margin-top: 2rem;
  }

  .tracks-header {
    display: grid;
    grid-template-columns: 40px 1fr 1fr 80px 80px;
    gap: 1rem;
    padding: 0.75rem 1rem;
    color: #606080;
    font-size: 0.75rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    border-bottom: 1px solid #2a2a4a;
  }

  .header-number {
    text-align: center;
  }

  .header-duration,
  .header-format {
    text-align: right;
  }

  .header-format {
    text-align: center;
  }

  .tracks-list {
    background: #16162a;
    border-radius: 0 0 8px 8px;
    border: 1px solid #2a2a4a;
    border-top: none;
    overflow: hidden;
  }

  .error {
    text-align: center;
    padding: 3rem;
    color: #ef4444;
  }

  .back-link {
    display: inline-block;
    margin-top: 1rem;
    color: #667eea;
    text-decoration: none;
  }

  .back-link:hover {
    text-decoration: underline;
  }
</style>
