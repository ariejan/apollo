<script lang="ts">
  import Header from '../lib/components/Header.svelte';
  import Loading from '../lib/components/Loading.svelte';
  import EmptyState from '../lib/components/EmptyState.svelte';
  import TrackRow from '../lib/components/TrackRow.svelte';
  import Pagination from '../lib/components/Pagination.svelte';
  import { getTracks } from '../lib/api/client';
  import type { Track } from '../lib/api/types';

  const LIMIT = 50;

  let tracks: Track[] = $state([]);
  let total = $state(0);
  let offset = $state(0);
  let loading = $state(true);
  let error: string | null = $state(null);

  $effect(() => {
    loadTracks(offset);
  });

  async function loadTracks(newOffset: number) {
    try {
      loading = true;
      error = null;
      const data = await getTracks(LIMIT, newOffset);
      tracks = data.items;
      total = data.total;
      offset = newOffset;
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to load tracks';
    } finally {
      loading = false;
    }
  }

  function handlePageChange(newOffset: number) {
    loadTracks(newOffset);
  }
</script>

<Header title="Tracks" />

<div class="content">
  {#if loading}
    <Loading />
  {:else if error}
    <div class="error">
      <p>Error: {error}</p>
      <button onclick={() => loadTracks(offset)}>Retry</button>
    </div>
  {:else if tracks.length === 0}
    <EmptyState message="No tracks in your library" icon="music" />
  {:else}
    <div class="track-count">
      {total} tracks
    </div>

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

    <Pagination {total} limit={LIMIT} {offset} onPageChange={handlePageChange} />
  {/if}
</div>

<style>
  .content {
    padding: 2rem;
  }

  .track-count {
    color: #808090;
    font-size: 0.875rem;
    margin-bottom: 1rem;
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
