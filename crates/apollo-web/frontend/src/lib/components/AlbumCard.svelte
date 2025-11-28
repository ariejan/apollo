<script lang="ts">
  import { link } from 'svelte-spa-router';
  import type { Album } from '../api/types';

  interface Props {
    album: Album;
  }

  let { album }: Props = $props();
</script>

<a href="/albums/{album.id}" use:link class="album-card">
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
    <div class="album-title">{album.title}</div>
    <div class="album-artist">{album.artist}</div>
    {#if album.year}
      <div class="album-year">{album.year}</div>
    {/if}
  </div>
</a>

<style>
  .album-card {
    display: flex;
    flex-direction: column;
    text-decoration: none;
    background: #1a1a2e;
    border-radius: 8px;
    overflow: hidden;
    transition: all 0.2s;
  }

  .album-card:hover {
    background: #2a2a4a;
    transform: translateY(-4px);
  }

  .album-cover {
    aspect-ratio: 1;
    background: #2a2a4a;
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
    width: 48px;
    height: 48px;
  }

  .album-info {
    padding: 1rem;
  }

  .album-title {
    color: #fff;
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    margin-bottom: 0.25rem;
  }

  .album-artist {
    color: #808090;
    font-size: 0.875rem;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .album-year {
    color: #606080;
    font-size: 0.75rem;
    margin-top: 0.25rem;
  }
</style>
