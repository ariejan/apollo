<script lang="ts">
  interface Props {
    total: number;
    limit: number;
    offset: number;
    onPageChange: (offset: number) => void;
  }

  let { total, limit, offset, onPageChange }: Props = $props();

  const currentPage = $derived(Math.floor(offset / limit) + 1);
  const totalPages = $derived(Math.ceil(total / limit));

  function goToPage(page: number) {
    const newOffset = (page - 1) * limit;
    onPageChange(newOffset);
  }

  function prevPage() {
    if (currentPage > 1) {
      goToPage(currentPage - 1);
    }
  }

  function nextPage() {
    if (currentPage < totalPages) {
      goToPage(currentPage + 1);
    }
  }
</script>

{#if totalPages > 1}
  <nav class="pagination" aria-label="Pagination">
    <button onclick={prevPage} disabled={currentPage === 1} aria-label="Previous page">
      <svg viewBox="0 0 24 24" fill="currentColor"><path d="M15.41 7.41L14 6l-6 6 6 6 1.41-1.41L10.83 12z"/></svg>
    </button>

    <span class="page-info">
      Page {currentPage} of {totalPages}
    </span>

    <button onclick={nextPage} disabled={currentPage === totalPages} aria-label="Next page">
      <svg viewBox="0 0 24 24" fill="currentColor"><path d="M10 6L8.59 7.41 13.17 12l-4.58 4.59L10 18l6-6z"/></svg>
    </button>
  </nav>
{/if}

<style>
  .pagination {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 1rem;
    padding: 1.5rem;
    border-top: 1px solid #2a2a4a;
  }

  button {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 36px;
    height: 36px;
    border: 1px solid #3a3a5a;
    background: #1a1a2e;
    color: #c0c0e0;
    border-radius: 6px;
    cursor: pointer;
    transition: all 0.15s;
  }

  button:hover:not(:disabled) {
    background: #2a2a4a;
    border-color: #667eea;
    color: #fff;
  }

  button:disabled {
    opacity: 0.3;
    cursor: not-allowed;
  }

  button svg {
    width: 20px;
    height: 20px;
  }

  .page-info {
    color: #808090;
    font-size: 0.875rem;
  }
</style>
