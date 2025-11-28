<script lang="ts">
  import Header from '../lib/components/Header.svelte';
  import Loading from '../lib/components/Loading.svelte';
  import EmptyState from '../lib/components/EmptyState.svelte';
  import TrackRow from '../lib/components/TrackRow.svelte';
  import Pagination from '../lib/components/Pagination.svelte';
  import { searchTracks } from '../lib/api/client';
  import type { Track } from '../lib/api/types';

  const LIMIT = 50;

  let query = $state('');
  let searchInput = $state('');
  let tracks: Track[] = $state([]);
  let total = $state(0);
  let offset = $state(0);
  let loading = $state(false);
  let error: string | null = $state(null);
  let hasSearched = $state(false);

  async function handleSearch(e: Event) {
    e.preventDefault();
    if (!searchInput.trim()) return;
    query = searchInput.trim();
    offset = 0;
    await doSearch();
  }

  async function doSearch() {
    if (!query) return;

    try {
      loading = true;
      error = null;
      hasSearched = true;
      const data = await searchTracks(query, LIMIT, offset);
      tracks = data.items;
      total = data.total;
    } catch (e) {
      error = e instanceof Error ? e.message : 'Search failed';
    } finally {
      loading = false;
    }
  }

  function handlePageChange(newOffset: number) {
    offset = newOffset;
    doSearch();
  }
</script>

<Header title="Search" />

<div class="content">
  <form class="search-form" onsubmit={handleSearch}>
    <div class="search-input-wrapper">
      <svg class="search-icon" viewBox="0 0 24 24" fill="currentColor">
        <path d="M15.5 14h-.79l-.28-.27C15.41 12.59 16 11.11 16 9.5 16 5.91 13.09 3 9.5 3S3 5.91 3 9.5 5.91 16 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5 14 7.01 14 9.5 11.99 14 9.5 14z"/>
      </svg>
      <input
        type="text"
        bind:value={searchInput}
        placeholder="Search for tracks, artists, or albums..."
        class="search-input"
      />
    </div>
    <button type="submit" class="search-button" disabled={!searchInput.trim()}>
      Search
    </button>
  </form>

  <div class="search-tips">
    <p>Search tips:</p>
    <ul>
      <li>Search by title, artist, or album name</li>
      <li>Use quotes for exact phrases: "dark side"</li>
      <li>Use field queries: artist:beatles album:abbey</li>
    </ul>
  </div>

  {#if loading}
    <Loading />
  {:else if error}
    <div class="error">
      <p>Error: {error}</p>
    </div>
  {:else if hasSearched}
    {#if tracks.length === 0}
      <EmptyState message="No tracks found for '{query}'" icon="search" />
    {:else}
      <div class="results-info">
        {total} results for "{query}"
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
  {/if}
</div>

<style>
  .content {
    padding: 2rem;
  }

  .search-form {
    display: flex;
    gap: 1rem;
    margin-bottom: 1.5rem;
  }

  .search-input-wrapper {
    flex: 1;
    position: relative;
  }

  .search-icon {
    position: absolute;
    left: 1rem;
    top: 50%;
    transform: translateY(-50%);
    width: 20px;
    height: 20px;
    color: #606080;
  }

  .search-input {
    width: 100%;
    padding: 0.875rem 1rem 0.875rem 3rem;
    background: #1a1a2e;
    border: 1px solid #2a2a4a;
    border-radius: 8px;
    color: #fff;
    font-size: 1rem;
  }

  .search-input:focus {
    outline: none;
    border-color: #667eea;
  }

  .search-input::placeholder {
    color: #606080;
  }

  .search-button {
    padding: 0.875rem 2rem;
    background: #667eea;
    border: none;
    border-radius: 8px;
    color: #fff;
    font-size: 1rem;
    font-weight: 500;
    cursor: pointer;
    transition: background 0.15s;
  }

  .search-button:hover:not(:disabled) {
    background: #7c8fef;
  }

  .search-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .search-tips {
    background: #1a1a2e;
    border: 1px solid #2a2a4a;
    border-radius: 8px;
    padding: 1rem 1.5rem;
    margin-bottom: 2rem;
    color: #808090;
    font-size: 0.875rem;
  }

  .search-tips p {
    margin: 0 0 0.5rem;
    color: #a0a0c0;
  }

  .search-tips ul {
    margin: 0;
    padding-left: 1.5rem;
  }

  .search-tips li {
    margin: 0.25rem 0;
  }

  .results-info {
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
</style>
