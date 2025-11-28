<script lang="ts">
  import type { Track } from '../api/types';
  import { formatDuration } from '../api/client';

  interface Props {
    track: Track;
    showAlbum?: boolean;
  }

  let { track, showAlbum = true }: Props = $props();
</script>

<div class="track-row">
  <div class="track-number">
    {#if track.track_number}
      {track.track_number}
    {:else}
      -
    {/if}
  </div>
  <div class="track-info">
    <div class="track-title">{track.title}</div>
    <div class="track-artist">{track.artist}</div>
  </div>
  {#if showAlbum}
    <div class="track-album">{track.album || '-'}</div>
  {/if}
  <div class="track-duration">{formatDuration(track.duration_ms)}</div>
  <div class="track-format">{track.format}</div>
</div>

<style>
  .track-row {
    display: grid;
    grid-template-columns: 40px 1fr 1fr 80px 80px;
    gap: 1rem;
    padding: 0.75rem 1rem;
    align-items: center;
    border-bottom: 1px solid #2a2a4a;
    transition: background 0.15s;
  }

  .track-row:hover {
    background: #1a1a2e;
  }

  .track-number {
    color: #606080;
    text-align: center;
    font-size: 0.875rem;
  }

  .track-info {
    min-width: 0;
  }

  .track-title {
    color: #fff;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .track-artist {
    color: #808090;
    font-size: 0.875rem;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .track-album {
    color: #808090;
    font-size: 0.875rem;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .track-duration {
    color: #808090;
    font-size: 0.875rem;
    text-align: right;
  }

  .track-format {
    color: #606080;
    font-size: 0.75rem;
    text-transform: uppercase;
    text-align: center;
    background: #2a2a4a;
    padding: 0.25rem 0.5rem;
    border-radius: 4px;
  }
</style>
