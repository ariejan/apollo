<script lang="ts">
  import Header from '../lib/components/Header.svelte';
  import Loading from '../lib/components/Loading.svelte';
  import EmptyState from '../lib/components/EmptyState.svelte';
  import AlbumCard from '../lib/components/AlbumCard.svelte';
  import Pagination from '../lib/components/Pagination.svelte';
  import { getAlbums } from '../lib/api/client';
  import type { Album } from '../lib/api/types';

  const LIMIT = 24;

  let albums: Album[] = $state([]);
  let total = $state(0);
  let offset = $state(0);
  let loading = $state(true);
  let error: string | null = $state(null);

  $effect(() => {
    loadAlbums(offset);
  });

  async function loadAlbums(newOffset: number) {
    try {
      loading = true;
      error = null;
      const data = await getAlbums(LIMIT, newOffset);
      albums = data.items;
      total = data.total;
      offset = newOffset;
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to load albums';
    } finally {
      loading = false;
    }
  }

  function handlePageChange(newOffset: number) {
    loadAlbums(newOffset);
  }
</script>

<Header title="Albums" />

<div class="content">
  {#if loading}
    <Loading />
  {:else if error}
    <div class="error">
      <p>Error: {error}</p>
      <button onclick={() => loadAlbums(offset)}>Retry</button>
    </div>
  {:else if albums.length === 0}
    <EmptyState message="No albums in your library" icon="album" />
  {:else}
    <div class="album-count">
      {total} albums
    </div>

    <div class="albums-grid">
      {#each albums as album}
        <AlbumCard {album} />
      {/each}
    </div>

    <Pagination {total} limit={LIMIT} {offset} onPageChange={handlePageChange} />
  {/if}
</div>

<style>
  .content {
    padding: 2rem;
  }

  .album-count {
    color: #808090;
    font-size: 0.875rem;
    margin-bottom: 1.5rem;
  }

  .albums-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
    gap: 1.5rem;
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
