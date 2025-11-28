<script lang="ts">
  import Header from '../lib/components/Header.svelte';
  import Loading from '../lib/components/Loading.svelte';
  import TrackRow from '../lib/components/TrackRow.svelte';
  import AlbumCard from '../lib/components/AlbumCard.svelte';
  import { link } from 'svelte-spa-router';
  import { getStats, getTracks, getAlbums, getHealth } from '../lib/api/client';
  import type { Stats, Track, Album, Health } from '../lib/api/types';

  let stats: Stats | null = $state(null);
  let recentTracks: Track[] = $state([]);
  let recentAlbums: Album[] = $state([]);
  let health: Health | null = $state(null);
  let loading = $state(true);
  let error: string | null = $state(null);

  $effect(() => {
    loadData();
  });

  async function loadData() {
    try {
      loading = true;
      error = null;

      const [statsData, tracksData, albumsData, healthData] = await Promise.all([
        getStats(),
        getTracks(5, 0),
        getAlbums(6, 0),
        getHealth(),
      ]);

      stats = statsData;
      recentTracks = tracksData.items;
      recentAlbums = albumsData.items;
      health = healthData;
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to load data';
    } finally {
      loading = false;
    }
  }
</script>

<Header title="Dashboard" />

<div class="content">
  {#if loading}
    <Loading />
  {:else if error}
    <div class="error">
      <p>Error: {error}</p>
      <button onclick={loadData}>Retry</button>
    </div>
  {:else}
    <div class="stats-grid">
      <div class="stat-card">
        <div class="stat-value">{stats?.track_count ?? 0}</div>
        <div class="stat-label">Tracks</div>
      </div>
      <div class="stat-card">
        <div class="stat-value">{stats?.album_count ?? 0}</div>
        <div class="stat-label">Albums</div>
      </div>
      <div class="stat-card">
        <div class="stat-value">{stats?.playlist_count ?? 0}</div>
        <div class="stat-label">Playlists</div>
      </div>
      <div class="stat-card">
        <div class="stat-value status-{health?.status}">{health?.status ?? 'unknown'}</div>
        <div class="stat-label">Server Status</div>
      </div>
    </div>

    {#if recentTracks.length > 0}
      <section class="section">
        <div class="section-header">
          <h2>Recent Tracks</h2>
          <a href="/tracks" use:link class="view-all">View all</a>
        </div>
        <div class="tracks-list">
          {#each recentTracks as track}
            <TrackRow {track} />
          {/each}
        </div>
      </section>
    {/if}

    {#if recentAlbums.length > 0}
      <section class="section">
        <div class="section-header">
          <h2>Recent Albums</h2>
          <a href="/albums" use:link class="view-all">View all</a>
        </div>
        <div class="albums-grid">
          {#each recentAlbums as album}
            <AlbumCard {album} />
          {/each}
        </div>
      </section>
    {/if}

    {#if recentTracks.length === 0 && recentAlbums.length === 0}
      <div class="empty-library">
        <svg viewBox="0 0 24 24" fill="currentColor"><path d="M12 3v10.55c-.59-.34-1.27-.55-2-.55-2.21 0-4 1.79-4 4s1.79 4 4 4 4-1.79 4-4V7h4V3h-6z"/></svg>
        <h2>Your library is empty</h2>
        <p>Import some music to get started!</p>
        <code>apollo import /path/to/music</code>
      </div>
    {/if}
  {/if}
</div>

<style>
  .content {
    padding: 2rem;
  }

  .stats-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
    gap: 1.5rem;
    margin-bottom: 2rem;
  }

  .stat-card {
    background: #1a1a2e;
    border-radius: 8px;
    padding: 1.5rem;
    text-align: center;
    border: 1px solid #2a2a4a;
  }

  .stat-value {
    font-size: 2rem;
    font-weight: 700;
    color: #667eea;
    margin-bottom: 0.5rem;
  }

  .stat-value.status-ok {
    color: #4ade80;
  }

  .stat-label {
    color: #808090;
    font-size: 0.875rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .section {
    margin-bottom: 2rem;
  }

  .section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 1rem;
  }

  .section-header h2 {
    margin: 0;
    font-size: 1.25rem;
    font-weight: 600;
    color: #fff;
  }

  .view-all {
    color: #667eea;
    text-decoration: none;
    font-size: 0.875rem;
  }

  .view-all:hover {
    text-decoration: underline;
  }

  .tracks-list {
    background: #16162a;
    border-radius: 8px;
    border: 1px solid #2a2a4a;
    overflow: hidden;
  }

  .albums-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
    gap: 1.5rem;
  }

  .empty-library {
    text-align: center;
    padding: 4rem 2rem;
    color: #606080;
  }

  .empty-library svg {
    width: 80px;
    height: 80px;
    opacity: 0.5;
    margin-bottom: 1.5rem;
  }

  .empty-library h2 {
    color: #a0a0c0;
    margin: 0 0 0.5rem;
  }

  .empty-library p {
    margin: 0 0 1.5rem;
  }

  .empty-library code {
    background: #1a1a2e;
    padding: 0.75rem 1.5rem;
    border-radius: 6px;
    font-family: 'Fira Code', monospace;
    color: #c0c0e0;
  }

  .error {
    text-align: center;
    padding: 3rem;
    color: #ef4444;
  }

  .error button {
    margin-top: 1rem;
    padding: 0.5rem 1rem;
    background: #667eea;
    color: #fff;
    border: none;
    border-radius: 6px;
    cursor: pointer;
  }
</style>
