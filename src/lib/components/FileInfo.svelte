<!-- FileInfo.svelte -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '../tauri';

  export let filePath: string;

  interface FileDetails {
    name: string;
    size: number;
    created: string;
    modified: string;
    type: string;
  }

  let fileDetails: FileDetails | null = null;
  let error: string | null = null;

  onMount(async () => {
    try {
      fileDetails = await invoke('get_file_info', { filePath });
    } catch (err) {
      error = `Failed to get file info: ${err}`;
    }
  });

  function formatBytes(bytes: number) {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  }

  function formatDate(date: string) {
    return new Date(date).toLocaleString();
  }
</script>

<div class="file-info">
  {#if error}
    <div class="error">{error}</div>
  {:else if fileDetails}
    <h4>File Details</h4>
    <div class="info-grid">
      <div class="info-item">
        <span class="info-label">Name</span>
        <span class="info-value">{fileDetails.name}</span>
      </div>
      <div class="info-item">
        <span class="info-label">Size</span>
        <span class="info-value">{formatBytes(fileDetails.size)}</span>
      </div>
      <div class="info-item">
        <span class="info-label">Type</span>
        <span class="info-value">{fileDetails.type}</span>
      </div>
      <div class="info-item">
        <span class="info-label">Created</span>
        <span class="info-value">{formatDate(fileDetails.created)}</span>
      </div>
      <div class="info-item">
        <span class="info-label">Modified</span>
        <span class="info-value">{formatDate(fileDetails.modified)}</span>
      </div>
      <div class="info-item full-width">
        <span class="info-label">Location</span>
        <span class="info-value path">{filePath}</span>
      </div>
    </div>
  {:else}
    <div class="loading">Loading file information...</div>
  {/if}
</div>

<style>
  .file-info {
    background: var(--bg-subtle);
    border-radius: 8px;
    padding: 1.5rem;
    margin: 1rem 0;
  }

  h4 {
    margin: 0 0 1rem 0;
    color: var(--text-color);
  }

  .info-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 1rem;
  }

  .info-item {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .info-item.full-width {
    grid-column: 1 / -1;
  }

  .info-label {
    font-size: 0.875rem;
    color: var(--text-muted);
  }

  .info-value {
    font-weight: 500;
    color: var(--text-color);
  }

  .info-value.path {
    word-break: break-all;
    font-family: monospace;
    font-size: 0.875rem;
    background: var(--bg-muted);
    padding: 0.5rem;
    border-radius: 4px;
  }

  .error {
    color: var(--error);
    padding: 1rem;
    background: var(--error-subtle);
    border-radius: 6px;
  }

  .loading {
    color: var(--text-muted);
    font-style: italic;
  }
</style>
