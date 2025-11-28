<script lang="ts">
  import Header from '../lib/components/Header.svelte';
  import Loading from '../lib/components/Loading.svelte';
  import EmptyState from '../lib/components/EmptyState.svelte';
  import { link } from 'svelte-spa-router';
  import { getPlaylists, createPlaylist, deletePlaylist, formatDate } from '../lib/api/client';
  import type { Playlist, CreatePlaylistRequest } from '../lib/api/types';

  let playlists: Playlist[] = $state([]);
  let loading = $state(true);
  let error: string | null = $state(null);
  let showCreateModal = $state(false);
  let creating = $state(false);

  // Create form state
  let newName = $state('');
  let newDescription = $state('');
  let newType: 'static' | 'smart' = $state('static');
  let newQuery = $state('');
  let newSortOrder = $state('');
  let newMaxTracks = $state('');

  $effect(() => {
    loadPlaylists();
  });

  async function loadPlaylists() {
    try {
      loading = true;
      error = null;
      playlists = await getPlaylists();
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to load playlists';
    } finally {
      loading = false;
    }
  }

  function openCreateModal() {
    showCreateModal = true;
    newName = '';
    newDescription = '';
    newType = 'static';
    newQuery = '';
    newSortOrder = '';
    newMaxTracks = '';
  }

  function closeCreateModal() {
    showCreateModal = false;
  }

  async function handleCreate(e: Event) {
    e.preventDefault();
    if (!newName.trim()) return;

    try {
      creating = true;
      const request: CreatePlaylistRequest = {
        name: newName.trim(),
        playlist_type: newType,
      };

      if (newDescription.trim()) {
        request.description = newDescription.trim();
      }

      if (newType === 'smart') {
        if (newQuery.trim()) {
          request.query = newQuery.trim();
        }
        if (newSortOrder.trim()) {
          request.sort_order = newSortOrder.trim();
        }
        if (newMaxTracks.trim()) {
          const max = parseInt(newMaxTracks.trim(), 10);
          if (!isNaN(max) && max > 0) {
            request.max_tracks = max;
          }
        }
      }

      await createPlaylist(request);
      closeCreateModal();
      await loadPlaylists();
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to create playlist';
    } finally {
      creating = false;
    }
  }

  async function handleDelete(playlist: Playlist) {
    if (!confirm(`Delete "${playlist.name}"?`)) return;

    try {
      await deletePlaylist(playlist.id);
      await loadPlaylists();
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to delete playlist';
    }
  }
</script>

<Header title="Playlists" />

<div class="content">
  <div class="toolbar">
    <button class="create-button" onclick={openCreateModal}>
      <svg viewBox="0 0 24 24" fill="currentColor"><path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z"/></svg>
      New Playlist
    </button>
  </div>

  {#if loading}
    <Loading />
  {:else if error}
    <div class="error">
      <p>Error: {error}</p>
      <button onclick={loadPlaylists}>Retry</button>
    </div>
  {:else if playlists.length === 0}
    <EmptyState message="No playlists yet" icon="playlist" />
  {:else}
    <div class="playlists-grid">
      {#each playlists as playlist}
        <div class="playlist-card">
          <a href="/playlists/{playlist.id}" use:link class="playlist-link">
            <div class="playlist-icon">
              {#if playlist.playlist_type === 'smart'}
                <svg viewBox="0 0 24 24" fill="currentColor"><path d="M14 2H6c-1.1 0-2 .9-2 2v16c0 1.1.9 2 2 2h12c1.1 0 2-.9 2-2V8l-6-6zm4 18H6V4h7v5h5v11zm-3-4h-2v2h-2v-2H9v-2h2v-2h2v2h2v2z"/></svg>
              {:else}
                <svg viewBox="0 0 24 24" fill="currentColor"><path d="M15 6H3v2h12V6zm0 4H3v2h12v-2zM3 16h8v-2H3v2zM17 6v8.18c-.31-.11-.65-.18-1-.18-1.66 0-3 1.34-3 3s1.34 3 3 3 3-1.34 3-3V8h3V6h-5z"/></svg>
              {/if}
            </div>
            <div class="playlist-info">
              <div class="playlist-name">{playlist.name}</div>
              {#if playlist.description}
                <div class="playlist-description">{playlist.description}</div>
              {/if}
              <div class="playlist-meta">
                <span class="playlist-type-badge" class:smart={playlist.playlist_type === 'smart'}>
                  {playlist.playlist_type}
                </span>
                <span>{playlist.track_count} tracks</span>
              </div>
            </div>
          </a>
          <button class="delete-button" onclick={() => handleDelete(playlist)} title="Delete playlist">
            <svg viewBox="0 0 24 24" fill="currentColor"><path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z"/></svg>
          </button>
        </div>
      {/each}
    </div>
  {/if}
</div>

{#if showCreateModal}
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div class="modal-overlay" onclick={closeCreateModal} role="presentation">
    <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
    <div class="modal" onclick={(e) => e.stopPropagation()} role="dialog" aria-modal="true" aria-labelledby="modal-title" tabindex="-1">
      <div class="modal-header">
        <h2 id="modal-title">Create Playlist</h2>
        <button class="close-button" onclick={closeCreateModal} aria-label="Close dialog">
          <svg viewBox="0 0 24 24" fill="currentColor"><path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/></svg>
        </button>
      </div>

      <form onsubmit={handleCreate}>
        <div class="form-group">
          <label for="name">Name</label>
          <input type="text" id="name" bind:value={newName} placeholder="My Playlist" required />
        </div>

        <div class="form-group">
          <label for="description">Description (optional)</label>
          <textarea id="description" bind:value={newDescription} placeholder="Playlist description..." rows="2"></textarea>
        </div>

        <div class="form-group">
          <span class="label-text">Type</span>
          <div class="type-buttons" role="group" aria-label="Playlist type">
            <button type="button" class:active={newType === 'static'} onclick={() => newType = 'static'}>
              Static
            </button>
            <button type="button" class:active={newType === 'smart'} onclick={() => newType = 'smart'}>
              Smart
            </button>
          </div>
        </div>

        {#if newType === 'smart'}
          <div class="form-group">
            <label for="query">Query</label>
            <input type="text" id="query" bind:value={newQuery} placeholder="artist:beatles year:1969" />
            <small>Use field queries like artist:, album:, year:, genre:</small>
          </div>

          <div class="form-group">
            <label for="sort">Sort Order (optional)</label>
            <select id="sort" bind:value={newSortOrder}>
              <option value="">Default</option>
              <option value="title">Title</option>
              <option value="artist">Artist</option>
              <option value="album">Album</option>
              <option value="year_asc">Year (oldest first)</option>
              <option value="year_desc">Year (newest first)</option>
              <option value="added_asc">Date added (oldest first)</option>
              <option value="added_desc">Date added (newest first)</option>
              <option value="random">Random</option>
            </select>
          </div>

          <div class="form-group">
            <label for="max">Max Tracks (optional)</label>
            <input type="number" id="max" bind:value={newMaxTracks} placeholder="100" min="1" />
          </div>
        {/if}

        <div class="form-actions">
          <button type="button" class="cancel-button" onclick={closeCreateModal}>Cancel</button>
          <button type="submit" class="submit-button" disabled={creating || !newName.trim()}>
            {creating ? 'Creating...' : 'Create'}
          </button>
        </div>
      </form>
    </div>
  </div>
{/if}

<style>
  .content {
    padding: 2rem;
  }

  .toolbar {
    margin-bottom: 1.5rem;
  }

  .create-button {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.75rem 1.25rem;
    background: #667eea;
    border: none;
    border-radius: 8px;
    color: #fff;
    font-size: 0.875rem;
    font-weight: 500;
    cursor: pointer;
    transition: background 0.15s;
  }

  .create-button:hover {
    background: #7c8fef;
  }

  .create-button svg {
    width: 18px;
    height: 18px;
  }

  .playlists-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
    gap: 1rem;
  }

  .playlist-card {
    display: flex;
    background: #1a1a2e;
    border: 1px solid #2a2a4a;
    border-radius: 8px;
    overflow: hidden;
    transition: border-color 0.15s;
  }

  .playlist-card:hover {
    border-color: #3a3a5a;
  }

  .playlist-link {
    flex: 1;
    display: flex;
    gap: 1rem;
    padding: 1rem;
    text-decoration: none;
  }

  .playlist-icon {
    width: 48px;
    height: 48px;
    background: #2a2a4a;
    border-radius: 6px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: #667eea;
    flex-shrink: 0;
  }

  .playlist-icon svg {
    width: 24px;
    height: 24px;
  }

  .playlist-info {
    flex: 1;
    min-width: 0;
  }

  .playlist-name {
    color: #fff;
    font-weight: 500;
    margin-bottom: 0.25rem;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .playlist-description {
    color: #808090;
    font-size: 0.875rem;
    margin-bottom: 0.5rem;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .playlist-meta {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    color: #606080;
    font-size: 0.75rem;
  }

  .playlist-type-badge {
    background: #2a2a4a;
    padding: 0.125rem 0.5rem;
    border-radius: 4px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .playlist-type-badge.smart {
    background: #3a3a5a;
    color: #667eea;
  }

  .delete-button {
    padding: 0 1rem;
    background: none;
    border: none;
    border-left: 1px solid #2a2a4a;
    color: #606080;
    cursor: pointer;
    transition: all 0.15s;
  }

  .delete-button:hover {
    background: #2a2a4a;
    color: #ef4444;
  }

  .delete-button svg {
    width: 20px;
    height: 20px;
  }

  .modal-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.7);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }

  .modal {
    background: #16162a;
    border: 1px solid #2a2a4a;
    border-radius: 12px;
    width: 100%;
    max-width: 480px;
    max-height: 90vh;
    overflow-y: auto;
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1.25rem 1.5rem;
    border-bottom: 1px solid #2a2a4a;
  }

  .modal-header h2 {
    margin: 0;
    font-size: 1.25rem;
    color: #fff;
  }

  .close-button {
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: none;
    border: none;
    color: #808090;
    cursor: pointer;
    border-radius: 6px;
  }

  .close-button:hover {
    background: #2a2a4a;
    color: #fff;
  }

  .close-button svg {
    width: 20px;
    height: 20px;
  }

  form {
    padding: 1.5rem;
  }

  .form-group {
    margin-bottom: 1.25rem;
  }

  label,
  .label-text {
    display: block;
    margin-bottom: 0.5rem;
    color: #a0a0c0;
    font-size: 0.875rem;
  }

  input[type="text"],
  input[type="number"],
  textarea,
  select {
    width: 100%;
    padding: 0.75rem;
    background: #1a1a2e;
    border: 1px solid #2a2a4a;
    border-radius: 6px;
    color: #fff;
    font-size: 0.875rem;
  }

  input:focus,
  textarea:focus,
  select:focus {
    outline: none;
    border-color: #667eea;
  }

  input::placeholder,
  textarea::placeholder {
    color: #606080;
  }

  small {
    display: block;
    margin-top: 0.25rem;
    color: #606080;
    font-size: 0.75rem;
  }

  .type-buttons {
    display: flex;
    gap: 0.5rem;
  }

  .type-buttons button {
    flex: 1;
    padding: 0.75rem;
    background: #1a1a2e;
    border: 1px solid #2a2a4a;
    border-radius: 6px;
    color: #808090;
    cursor: pointer;
    transition: all 0.15s;
  }

  .type-buttons button:hover {
    border-color: #3a3a5a;
    color: #c0c0e0;
  }

  .type-buttons button.active {
    background: #667eea;
    border-color: #667eea;
    color: #fff;
  }

  .form-actions {
    display: flex;
    gap: 0.75rem;
    justify-content: flex-end;
    margin-top: 1.5rem;
  }

  .cancel-button {
    padding: 0.75rem 1.25rem;
    background: #2a2a4a;
    border: none;
    border-radius: 6px;
    color: #c0c0e0;
    cursor: pointer;
  }

  .cancel-button:hover {
    background: #3a3a5a;
  }

  .submit-button {
    padding: 0.75rem 1.25rem;
    background: #667eea;
    border: none;
    border-radius: 6px;
    color: #fff;
    font-weight: 500;
    cursor: pointer;
  }

  .submit-button:hover:not(:disabled) {
    background: #7c8fef;
  }

  .submit-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
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
