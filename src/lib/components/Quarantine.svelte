<!-- Quarantine.svelte -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '../tauri';
  import type { QuarantinedFile } from '../types';

  let quarantinedFiles: QuarantinedFile[] = [];
  let isLoading = true;
  let error: string | null = null;
  let selectedFile: QuarantinedFile | null = null;

  onMount(async () => {
    await loadQuarantinedFiles();
  });

  async function loadQuarantinedFiles() {
    try {
      isLoading = true;
      error = null;
      quarantinedFiles = await invoke('get_quarantined_files');
    } catch (err) {
      error = `Failed to load quarantined files: ${err}`;
    } finally {
      isLoading = false;
    }
  }

  async function restoreFile(file: QuarantinedFile) {
    if (!confirm(`Are you sure you want to restore "${file.file_name}"? This file was quarantined due to potential security risks.`)) {
      return;
    }

    try {
      await invoke('restore_file', { quarantineId: file.id });
      await loadQuarantinedFiles();
    } catch (err) {
      error = `Failed to restore file: ${err}`;
    }
  }

  async function deleteFile(file: QuarantinedFile) {
    if (!confirm(`Are you sure you want to permanently delete "${file.file_name}"? This action cannot be undone.`)) {
      return;
    }

    try {
      await invoke('delete_file', { quarantineId: file.id });
      await loadQuarantinedFiles();
    } catch (err) {
      error = `Failed to delete file: ${err}`;
    }
  }

  function formatDate(date: string) {
    return new Date(date).toLocaleString();
  }

  function formatBytes(bytes: number) {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  }

  function selectFile(file: QuarantinedFile) {
    selectedFile = selectedFile?.id === file.id ? null : file;
  }
</script>

<div class="quarantine-container">
  <h2>Quarantined Files</h2>

  {#if error}
    <div class="error-message">
      {error}
      <button class="retry-button" on:click={loadQuarantinedFiles}>
        Retry
      </button>
    </div>
  {/if}

  {#if isLoading}
    <div class="loading">
      <div class="spinner"></div>
      <p>Loading quarantined files...</p>
    </div>
  {:else if quarantinedFiles.length === 0}
    <div class="empty-state">
      <div class="icon">🛡️</div>
      <h3>No Quarantined Files</h3>
      <p>Files that are identified as potentially malicious will appear here.</p>
    </div>
  {:else}
    <div class="files-list">
      {#each quarantinedFiles as file (file.id)}
        <div
          class="file-item {selectedFile?.id === file.id ? 'selected' : ''}"
          on:click={() => selectFile(file)}
        >
          <div class="file-info">
            <div class="file-name">
              <span class="icon">📄</span>
              {file.file_name}
            </div>
            <div class="file-details">
              <span class="detail">
                <span class="label">Size:</span>
                {formatBytes(file.file_size)}
              </span>
              <span class="detail">
                <span class="label">Detection Rate:</span>
                {file.detection_count}/{file.total_engines}
              </span>
              <span class="detail">
                <span class="label">Quarantined:</span>
                {formatDate(file.quarantine_date)}
              </span>
            </div>
            {#if selectedFile?.id === file.id}
              <div class="file-path">
                <span class="label">Original Location:</span>
                {file.original_path}
              </div>
            {/if}
          </div>
          <div class="file-actions">
            <button
              class="restore-button"
              on:click|stopPropagation={() => restoreFile(file)}
              title="Restore file to its original location"
            >
              Restore
            </button>
            <button
              class="delete-button"
              on:click|stopPropagation={() => deleteFile(file)}
              title="Permanently delete file"
            >
              Delete
            </button>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .quarantine-container {
    max-width: 800px;
    margin: 0 auto;
    padding: 2rem;
  }

  h2 {
    color: var(--text-color);
    margin-bottom: 2rem;
    text-align: center;
  }

  .error-message {
    background: var(--error-subtle);
    color: var(--error);
    padding: 1rem;
    border-radius: 8px;
    margin-bottom: 1rem;
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .retry-button {
    background: var(--error);
    color: white;
    border: none;
    padding: 0.5rem 1rem;
    border-radius: 4px;
    cursor: pointer;
  }

  .loading {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 3rem;
  }

  .spinner {
    width: 40px;
    height: 40px;
    border: 4px solid var(--border-color);
    border-radius: 50%;
    border-top-color: var(--primary-color);
    animation: spin 1s ease-in-out infinite;
    margin-bottom: 1rem;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .empty-state {
    text-align: center;
    padding: 3rem;
    background: var(--bg-subtle);
    border-radius: 8px;
  }

  .empty-state .icon {
    font-size: 3rem;
    margin-bottom: 1rem;
    color: var(--text-muted);
  }

  .empty-state h3 {
    color: var(--text-color);
    margin-bottom: 0.5rem;
  }

  .empty-state p {
    color: var(--text-muted);
    margin: 0;
  }

  .files-list {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .file-item {
    background: var(--card-bg);
    border-radius: 8px;
    padding: 1.5rem;
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 1rem;
    cursor: pointer;
    transition: all 0.2s;
    border: 1px solid var(--border-color);
  }

  .file-item:hover {
    transform: translateY(-2px);
    box-shadow: var(--shadow);
  }

  .file-item.selected {
    border-color: var(--primary-color);
    background: var(--bg-subtle);
  }

  .file-info {
    flex: 1;
  }

  .file-name {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-weight: 500;
    color: var(--text-color);
    margin-bottom: 0.5rem;
  }

  .file-details {
    display: flex;
    flex-wrap: wrap;
    gap: 1rem;
    font-size: 0.875rem;
    color: var(--text-muted);
  }

  .detail {
    display: flex;
    align-items: center;
    gap: 0.25rem;
  }

  .label {
    color: var(--text-muted);
  }

  .file-path {
    margin-top: 0.5rem;
    font-size: 0.875rem;
    word-break: break-all;
    padding: 0.5rem;
    background: var(--bg-muted);
    border-radius: 4px;
  }

  .file-actions {
    display: flex;
    gap: 0.5rem;
  }

  .restore-button,
  .delete-button {
    padding: 0.5rem 1rem;
    border: none;
    border-radius: 4px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
  }

  .restore-button {
    background: var(--warning);
    color: white;
  }

  .delete-button {
    background: var(--error);
    color: white;
  }

  .restore-button:hover,
  .delete-button:hover {
    opacity: 0.9;
    transform: translateY(-1px);
  }

  @media (max-width: 640px) {
    .quarantine-container {
      padding: 1rem;
    }

    .file-item {
      flex-direction: column;
    }

    .file-actions {
      width: 100%;
      justify-content: stretch;
    }

    .restore-button,
    .delete-button {
      flex: 1;
    }

    .file-details {
      flex-direction: column;
      gap: 0.5rem;
    }
  }
</style>
