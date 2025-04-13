<script>
  // Import global styles
  import '../app.css';
  import { onMount } from 'svelte';

  let loaded = false;

  onMount(() => {
    // Set loaded to true after a short delay to ensure everything is initialized
    setTimeout(() => {
      loaded = true;
    }, 100);
  });
</script>

{#if loaded}
  <slot />
{:else}
  <div class="loading-container">
    <div class="loading-spinner"></div>
    <p>Loading application...</p>
  </div>
{/if}

<style>
  .loading-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100vh;
    background-color: #f8fafc;
  }

  .loading-spinner {
    width: 50px;
    height: 50px;
    border: 5px solid rgba(0, 0, 0, 0.1);
    border-radius: 50%;
    border-top-color: #2563eb;
    animation: spin 1s ease-in-out infinite;
    margin-bottom: 1rem;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  p {
    color: #1e293b;
    font-size: 1.1rem;
  }
</style>