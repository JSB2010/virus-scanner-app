<!-- History.svelte -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke, open, save } from '../tauri';
  import type { ScanResult } from '../types';

  let scanHistory: ScanResult[] = [];
  let isLoading = true;
  let error: string | null = null;
  let searchQuery = '';
  let statusFilter: string = 'all';
  let sortBy: 'date' | 'name' | 'status' = 'date';
  let sortDirection: 'asc' | 'desc' = 'desc';
  let selectedResult: ScanResult | null = null;

  onMount(async () => {
    await loadHistory();
  });

  async function loadHistory() {
    try {
      isLoading = true;
      error = null;
      scanHistory = await invoke('get_scan_history');
    } catch (err) {
      error = `Failed to load scan history: ${err}`;
    } finally {
      isLoading = false;
    }
  }

  async function exportHistory() {
    try {
      const filePath = await save({
        filters: [{
          name: 'JSON',
          extensions: ['json']
        }]
      });

      if (filePath) {
        await invoke('export_scan_history', { filePath });
        await invoke('show_notification', {
          title: 'Export Complete',
          body: 'Scan history has been exported successfully.'
        });
      }
    } catch (err) {
      error = `Failed to export history: ${err}`;
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

  $: filteredHistory = scanHistory
    .filter(result => {
      const matchesSearch = searchQuery === '' ||
        result.file_name.toLowerCase().includes(searchQuery.toLowerCase()) ||
        result.file_path.toLowerCase().includes(searchQuery.toLowerCase());

      const matchesStatus = statusFilter === 'all' ||
        result.status.toLowerCase() === statusFilter.toLowerCase();

      return matchesSearch && matchesStatus;
    })
    .sort((a, b) => {
      if (sortBy === 'date') {
        return sortDirection === 'desc'
          ? new Date(b.scan_date).getTime() - new Date(a.scan_date).getTime()
          : new Date(a.scan_date).getTime() - new Date(b.scan_date).getTime();
      }
      if (sortBy === 'name') {
        return sortDirection === 'desc'
          ? b.file_name.localeCompare(a.file_name)
          : a.file_name.localeCompare(b.file_name);
      }
      // Sort by status
      return sortDirection === 'desc'
        ? b.status.localeCompare(a.status)
        : a.status.localeCompare(b.status);
    });
</script>

<div class="history-container">
  <div class="controls">
    <div class="search-bar">
      <input
        type="text"
        placeholder="Search files..."
        bind:value={searchQuery}
      />
    </div>

    <div class="filters">
      <select bind:value={statusFilter}>
        <option value="all">All Status</option>
        <option value="clean">Clean</option>
        <option value="suspicious">Suspicious</option>
        <option value="malicious">Malicious</option>
      </select>

      <select bind:value={sortBy}>
        <option value="date">Sort by Date</option>
        <option value="name">Sort by Name</option>
        <option value="status">Sort by Status</option>
      </select>

      <button
        class="sort-direction"
        on:click={() => sortDirection = sortDirection === 'asc' ? 'desc' : 'asc'}
      >
        {sortDirection === 'asc' ? '↑' : '↓'}
      </button>

      <button class="export-button" on:click={exportHistory}>
        Export History
      </button>
    </div>
  </div>

  {#if error}
    <div class="error-message">
      {error}
      <button class="retry-button" on:click={loadHistory}>
        Retry
      </button>
    </div>
  {/if}

  {#if isLoading}
    <div class="loading">
      <div class="spinner"></div>
      <p>Loading scan history...</p>
    </div>
  {:else if filteredHistory.length === 0}
    <div class="empty-state">
      <div class="icon">📊</div>
      <h3>No Scan History</h3>
      <p>Your scan history will appear here after scanning files.</p>
    </div>
  {:else}
    <div class="history-grid">
      {#each filteredHistory as result (result.id)}
        <div
          class="history-item {result.status.toLowerCase()} {selectedResult?.id === result.id ? 'selected' : ''}"
          on:click={() => selectedResult = selectedResult?.id === result.id ? null : result}
        >
          <div class="item-header">
            <div class="file-info">
              <span class="icon">📄</span>
              <span class="file-name">{result.file_name}</span>
            </div>
            <div class="status-badge {result.status.toLowerCase()}">
              {result.status}
            </div>
          </div>

          <div class="item-details">
            <div class="detail">
              <span class="label">Scanned:</span>
              {formatDate(result.scan_date)}
            </div>
            <div class="detail">
              <span class="label">Size:</span>
              {formatBytes(result.file_size)}
            </div>
            {#if result.detection_count !== undefined && result.total_engines !== undefined}
              <div class="detail">
                <span class="label">Detections:</span>
                {result.detection_count}/{result.total_engines}
              </div>
            {/if}
          </div>

          {#if selectedResult?.id === result.id}
            <div class="expanded-details">
              <div class="detail-row">
                <span class="label">File Path:</span>
                <span class="value path">{result.file_path}</span>
              </div>
              <div class="detail-row">
                <span class="label">SHA-256:</span>
                <span class="value hash">{result.file_hash}</span>
              </div>
              {#if result.vendor_results}
                <div class="vendors-summary">
                  <h4>Antivirus Results</h4>
                  <div class="vendors-grid">
                    {#each Object.entries(result.vendor_results) as [vendor, data]}
                      {#if data.detected}
                        <div class="vendor-result">
                          <span class="vendor-name">{vendor}</span>
                          <span class="detection">{data.result || 'Detected'}</span>
                        </div>
                      {/if}
                    {/each}
                  </div>
                </div>
              {/if}
              {#if result.permalink}
                <a
                  href={result.permalink}
                  target="_blank"
                  rel="noopener noreferrer"
                  class="vt-link"
                >
                  View on VirusTotal
                </a>
              {/if}
            </div>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .history-container {
    max-width: 1200px;
    margin: 0 auto;
    padding: 2rem;
  }

  .controls {
    margin-bottom: 2rem;
  }

  .search-bar {
    margin-bottom: 1rem;
  }

  .search-bar input {
    width: 100%;
    padding: 0.75rem;
    border: 1px solid var(--border-color);
    border-radius: 6px;
    background: var(--bg-subtle);
    color: var(--text-color);
  }

  .filters {
    display: flex;
    gap: 1rem;
    align-items: center;
    flex-wrap: wrap;
  }

  select {
    padding: 0.5rem;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    background: var(--bg-subtle);
    color: var(--text-color);
  }

  .sort-direction {
    padding: 0.5rem 1rem;
    background: var(--bg-subtle);
    border: 1px solid var(--border-color);
    border-radius: 4px;
    cursor: pointer;
  }

  .export-button {
    margin-left: auto;
    padding: 0.5rem 1rem;
    background: var(--primary-color);
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
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

  .history-grid {
    display: grid;
    gap: 1rem;
  }

  .history-item {
    background: var(--card-bg);
    border-radius: 8px;
    padding: 1.5rem;
    cursor: pointer;
    transition: all 0.2s;
    border: 1px solid var(--border-color);
  }

  .history-item:hover {
    transform: translateY(-2px);
    box-shadow: var(--shadow);
  }

  .history-item.selected {
    border-color: var(--primary-color);
  }

  .history-item.clean {
    border-left: 4px solid var(--success);
  }

  .history-item.suspicious {
    border-left: 4px solid var(--warning);
  }

  .history-item.malicious {
    border-left: 4px solid var(--error);
  }

  .item-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
  }

  .file-info {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .file-name {
    font-weight: 500;
    color: var(--text-color);
  }

  .status-badge {
    padding: 0.25rem 0.75rem;
    border-radius: 12px;
    font-size: 0.875rem;
    font-weight: 500;
  }

  .status-badge.clean {
    background: var(--success-subtle);
    color: var(--success);
  }

  .status-badge.suspicious {
    background: var(--warning-subtle);
    color: var(--warning);
  }

  .status-badge.malicious {
    background: var(--error-subtle);
    color: var(--error);
  }

  .item-details {
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

  .expanded-details {
    margin-top: 1rem;
    padding-top: 1rem;
    border-top: 1px solid var(--border-color);
  }

  .detail-row {
    margin-bottom: 0.5rem;
  }

  .value {
    color: var(--text-color);
    word-break: break-all;
  }

  .value.path,
  .value.hash {
    font-family: monospace;
    font-size: 0.875rem;
    background: var(--bg-subtle);
    padding: 0.5rem;
    border-radius: 4px;
    display: block;
    margin-top: 0.25rem;
  }

  .vendors-summary {
    margin-top: 1rem;
  }

  .vendors-summary h4 {
    margin: 0 0 0.5rem 0;
    font-size: 1rem;
    color: var(--text-color);
  }

  .vendors-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 0.5rem;
    max-height: 200px;
    overflow-y: auto;
    padding: 0.5rem;
    background: var(--bg-subtle);
    border-radius: 4px;
  }

  .vendor-result {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.5rem;
    background: var(--error-subtle);
    border-radius: 4px;
    font-size: 0.875rem;
  }

  .vendor-name {
    font-weight: 500;
  }

  .detection {
    color: var(--error);
  }

  .vt-link {
    display: inline-block;
    margin-top: 1rem;
    color: var(--primary-color);
    text-decoration: none;
  }

  .vt-link:hover {
    text-decoration: underline;
  }

  @media (max-width: 768px) {
    .history-container {
      padding: 1rem;
    }

    .filters {
      flex-direction: column;
      align-items: stretch;
    }

    .export-button {
      margin: 0;
    }

    .vendors-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
