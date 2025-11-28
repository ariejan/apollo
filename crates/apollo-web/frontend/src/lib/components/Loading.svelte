<div class="loading">
  <div class="spinner"></div>
  <span>Loading...</span>
</div>

<style>
  .loading {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 3rem;
    gap: 1rem;
    color: #808090;
  }

  .spinner {
    width: 40px;
    height: 40px;
    border: 3px solid #2a2a4a;
    border-top-color: #667eea;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
