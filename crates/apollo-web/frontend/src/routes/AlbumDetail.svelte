<script lang="ts">
  import Header from '../lib/components/Header.svelte';
  import Loading from '../lib/components/Loading.svelte';
  import TrackRow from '../lib/components/TrackRow.svelte';
  import { link } from 'svelte-spa-router';
  import { getAlbum, getAlbumTracks, formatDuration } from '../lib/api/client';
  import type { Album, Track } from '../lib/api/types';

  interface Props {
    params: { id: string };
  }

  let { params }: Props = $props();

  let album: Album | null = $state(null);
  let tracks: Track[] = $state([]);
  let loading = $state(true);
  let error: string | null = $state(null);

  $effect(() => {
    loadAlbum(params.id);
  });

  async function loadAlbum(id: string) {
    try {
      loading = true;
      error = null;
      const [albumData, tracksData] = await Promise.all([
        getAlbum(id),
        getAlbumTracks(id),
      ]);
      album = albumData;
      tracks = tracksData;
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to load album';
    } finally {
      loading = false;
    }
  }

  const totalDuration = $derived(
    tracks.reduce((sum, t) => sum + (t.duration_ms ?? 0), 0)
  );
</script>

<Header title={album?.title ?? 'Album'} />

<div class="content">
  {#if loading}
    <Loading />
  {:else if error}
    <div class="error">
      <p>Error: {error}</p>
      <a href="/albums" use:link class="back-link">Back to Albums</a>
    </div>
  {:else if album}
    <div class="album-header">
      <div class="album-cover">
        {#if album.cover_art_path}
          <img src={album.cover_art_path} alt={album.title} />
        {:else}
          <div class="album-placeholder">
            <svg viewBox="0 0 24 24" fill="currentColor">
              <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 18c-4.41 0-8-3.59-8-8s3.59-8 8-8 8 3.59 8 8-3.59 8-8 8zm0-12.5c-2.49 0-4.5 2.01-4.5 4.5s2.01 4.5 4.5 4.5 4.5-2.01 4.5-4.5-2.01-4.5-4.5-4.5zm0 5.5c-.55 0-1-.45-1-1s.45-1 1-1 1 .45 1 1-.45 1-1 1z"/>
            </svg>
          </div>
        {/if}
      </div>
      <div class="album-info">
        <span class="album-type">Album</span>
        <h1>{album.title}</h1>
        <p class="album-artist">{album.artist}</p>
        <div class="album-meta">
          {#if album.year}
            <span>{album.year}</span>
            <span class="separator">-</span>
          {/if}
          <span>{tracks.length} tracks</span>
          <span class="separator">-</span>
          <span>{formatDuration(totalDuration)}</span>
        </div>
        {#if album.genres.length > 0}
          <div class="album-genres">
            {#each album.genres as genre}
              <span class="genre-tag">{genre}</span>
            {/each}
          </div>
        {/if}
      </div>
    </div>

    <div class="tracks-section">
      <div class="tracks-header">
        <div class="header-number">#</div>
        <div class="header-title">Title / Artist</div>
        <div class="header-duration">Duration</div>
        <div class="header-format">Format</div>
      </div>

      <div class="tracks-list">
        {#each tracks as track}
          <TrackRow {track} showAlbum={false} />
        {/each}
      </div>
    </div>
  {/if}
</div>

<style>
  .content {
    padding: 2rem;
  }

  .album-header {
    display: flex;
    gap: 2rem;
    margin-bottom: 2rem;
  }

  .album-cover {
    width: 240px;
    height: 240px;
    flex-shrink: 0;
    background: #2a2a4a;
    border-radius: 8px;
    overflow: hidden;
  }

  .album-cover img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .album-placeholder {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    color: #4a4a6a;
  }

  .album-placeholder svg {
    width: 80px;
    height: 80px;
  }

  .album-info {
    display: flex;
    flex-direction: column;
    justify-content: flex-end;
  }

  .album-type {
    font-size: 0.75rem;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: #808090;
    margin-bottom: 0.5rem;
  }

  .album-info h1 {
    margin: 0 0 0.5rem;
    font-size: 2rem;
    font-weight: 700;
    color: #fff;
  }

  .album-artist {
    margin: 0 0 0.75rem;
    font-size: 1.125rem;
    color: #c0c0e0;
  }

  .album-meta {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    color: #808090;
    font-size: 0.875rem;
    margin-bottom: 1rem;
  }

  .separator {
    color: #4a4a6a;
  }

  .album-genres {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
  }

  .genre-tag {
    background: #2a2a4a;
    padding: 0.25rem 0.75rem;
    border-radius: 12px;
    font-size: 0.75rem;
    color: #c0c0e0;
  }

  .tracks-section {
    margin-top: 2rem;
  }

  .tracks-header {
    display: grid;
    grid-template-columns: 40px 1fr 80px 80px;
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

  .header-duration {
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

  .tracks-list :global(.track-row) {
    grid-template-columns: 40px 1fr 80px 80px;
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
