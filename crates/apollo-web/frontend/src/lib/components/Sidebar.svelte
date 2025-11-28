<script lang="ts">
  import { link } from 'svelte-spa-router';
  import { getStats } from '../api/client';
  import type { Stats } from '../api/types';

  let stats: Stats | null = $state(null);

  $effect(() => {
    getStats().then(s => stats = s).catch(() => {});
  });
</script>

<aside class="sidebar">
  <div class="logo">
    <img src="/apollo.svg" alt="Apollo" />
    <span>Apollo</span>
  </div>

  <nav>
    <ul>
      <li>
        <a href="/" use:link>
          <svg viewBox="0 0 24 24" fill="currentColor"><path d="M10 20v-6h4v6h5v-8h3L12 3 2 12h3v8z"/></svg>
          Dashboard
        </a>
      </li>
      <li>
        <a href="/tracks" use:link>
          <svg viewBox="0 0 24 24" fill="currentColor"><path d="M12 3v10.55c-.59-.34-1.27-.55-2-.55-2.21 0-4 1.79-4 4s1.79 4 4 4 4-1.79 4-4V7h4V3h-6z"/></svg>
          Tracks
          {#if stats}<span class="badge">{stats.track_count}</span>{/if}
        </a>
      </li>
      <li>
        <a href="/albums" use:link>
          <svg viewBox="0 0 24 24" fill="currentColor"><path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 18c-4.41 0-8-3.59-8-8s3.59-8 8-8 8 3.59 8 8-3.59 8-8 8zm0-12.5c-2.49 0-4.5 2.01-4.5 4.5s2.01 4.5 4.5 4.5 4.5-2.01 4.5-4.5-2.01-4.5-4.5-4.5zm0 5.5c-.55 0-1-.45-1-1s.45-1 1-1 1 .45 1 1-.45 1-1 1z"/></svg>
          Albums
          {#if stats}<span class="badge">{stats.album_count}</span>{/if}
        </a>
      </li>
      <li>
        <a href="/playlists" use:link>
          <svg viewBox="0 0 24 24" fill="currentColor"><path d="M15 6H3v2h12V6zm0 4H3v2h12v-2zM3 16h8v-2H3v2zM17 6v8.18c-.31-.11-.65-.18-1-.18-1.66 0-3 1.34-3 3s1.34 3 3 3 3-1.34 3-3V8h3V6h-5z"/></svg>
          Playlists
          {#if stats}<span class="badge">{stats.playlist_count}</span>{/if}
        </a>
      </li>
      <li>
        <a href="/search" use:link>
          <svg viewBox="0 0 24 24" fill="currentColor"><path d="M15.5 14h-.79l-.28-.27C15.41 12.59 16 11.11 16 9.5 16 5.91 13.09 3 9.5 3S3 5.91 3 9.5 5.91 16 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5 14 7.01 14 9.5 11.99 14 9.5 14z"/></svg>
          Search
        </a>
      </li>
    </ul>
  </nav>
</aside>

<style>
  .sidebar {
    width: 240px;
    background: #1a1a2e;
    height: 100vh;
    position: fixed;
    left: 0;
    top: 0;
    display: flex;
    flex-direction: column;
    border-right: 1px solid #2a2a4a;
  }

  .logo {
    padding: 1.5rem;
    display: flex;
    align-items: center;
    gap: 0.75rem;
    border-bottom: 1px solid #2a2a4a;
  }

  .logo img {
    width: 32px;
    height: 32px;
  }

  .logo span {
    font-size: 1.25rem;
    font-weight: 600;
    color: #fff;
  }

  nav {
    flex: 1;
    padding: 1rem 0;
  }

  ul {
    list-style: none;
    padding: 0;
    margin: 0;
  }

  li a {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.75rem 1.5rem;
    color: #a0a0c0;
    text-decoration: none;
    transition: all 0.2s;
  }

  li a:hover {
    background: #2a2a4a;
    color: #fff;
  }

  li a svg {
    width: 20px;
    height: 20px;
    flex-shrink: 0;
  }

  .badge {
    margin-left: auto;
    background: #3a3a5a;
    padding: 0.125rem 0.5rem;
    border-radius: 10px;
    font-size: 0.75rem;
    color: #c0c0e0;
  }
</style>
