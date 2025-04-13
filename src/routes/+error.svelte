<script>
  import { page } from '$app/stores'; // Using page store for error handling
  import { onMount } from 'svelte';

  let retryCount = 0;
  const maxRetries = 3;

  onMount(() => {
    // Auto-retry a few times if we get a 500 error
    if ($page.status === 500 && retryCount < maxRetries) {
      retryCount++;
      console.log(`Auto-retrying (${retryCount}/${maxRetries})...`);
      setTimeout(() => {
        window.location.reload();
      }, 1000);
    }
  });

  function handleRetry() {
    window.location.reload();
  }

  function handleGoHome() {
    window.location.href = '/';
  }
</script>

<div class="error-container">
  <div class="error-card">
    <div class="error-icon">⚠️</div>
    <h1>{$page.status || 500}: {$page.error?.message || 'An error occurred'}</h1>
    <p>Sorry, something went wrong. Please try refreshing the page.</p>
    <div class="button-group">
      <button on:click={handleRetry}>Refresh Page</button>
      <button on:click={handleGoHome}>Go to Home</button>
    </div>
  </div>
</div>

<style>
  .error-container {
    display: flex;
    justify-content: center;
    align-items: center;
    min-height: 100vh;
    padding: 2rem;
    background-color: var(--bg-color, #f8fafc);
  }

  .error-card {
    background: var(--card-bg, white);
    padding: 2rem;
    border-radius: 12px;
    box-shadow: var(--shadow, 0 4px 6px rgba(0, 0, 0, 0.1));
    text-align: center;
    max-width: 500px;
    width: 100%;
  }

  .error-icon {
    font-size: 4rem;
    margin-bottom: 1rem;
  }

  h1 {
    color: var(--text-color, #1e293b);
    margin-bottom: 1rem;
    font-size: 1.5rem;
  }

  p {
    color: var(--text-muted, #64748b);
    margin-bottom: 2rem;
  }

  .button-group {
    display: flex;
    gap: 1rem;
    justify-content: center;
  }

  button {
    background: var(--primary-color, #2563eb);
    color: white;
    padding: 0.75rem 1.5rem;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-weight: 500;
    transition: all 0.2s;
  }

  button:hover {
    background: var(--primary-hover, #1d4ed8);
    transform: translateY(-2px);
  }
</style>
