<script lang="ts">
  import Router from 'svelte-spa-router';
  import Sidebar from './lib/components/Sidebar.svelte';
  import Dashboard from './routes/Dashboard.svelte';
  import Tracks from './routes/Tracks.svelte';
  import Albums from './routes/Albums.svelte';
  import AlbumDetail from './routes/AlbumDetail.svelte';
  import Search from './routes/Search.svelte';
  import Playlists from './routes/Playlists.svelte';
  import PlaylistDetail from './routes/PlaylistDetail.svelte';

  const routes = {
    '/': Dashboard,
    '/tracks': Tracks,
    '/albums': Albums,
    '/albums/:id': AlbumDetail,
    '/search': Search,
    '/playlists': Playlists,
    '/playlists/:id': PlaylistDetail,
  };
</script>

<div class="app">
  <Sidebar />
  <main class="main-content">
    <Router {routes} />
  </main>
</div>

<style>
  :global(*) {
    box-sizing: border-box;
  }

  :global(body) {
    margin: 0;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen,
      Ubuntu, Cantarell, 'Fira Sans', 'Droid Sans', 'Helvetica Neue', sans-serif;
    background: #0d0d1a;
    color: #e0e0f0;
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
  }

  .app {
    display: flex;
    min-height: 100vh;
  }

  .main-content {
    flex: 1;
    margin-left: 240px;
    min-height: 100vh;
    background: #12122a;
  }
</style>
