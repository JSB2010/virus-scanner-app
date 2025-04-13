<!-- ScanResults.svelte -->
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { ScanResult, VendorResult } from '../types';

  export let results: ScanResult;
  export let settings = {
    auto_quarantine_malicious: false
  };

  const dispatch = createEventDispatcher();
  let selectedFilter: 'all' | 'detected' = 'all';

  $: detectionRate = results.detection_count && results.total_engines
    ? (results.detection_count / results.total_engines) * 100
    : 0;

  $: threatLevel = detectionRate > 50 
    ? 'High' 
    : detectionRate > 20 
    ? 'Medium' 
    : detectionRate > 0 
    ? 'Low' 
    : 'Clean';

  $: filteredResults = results.vendor_results ? 
    Object.entries(results.vendor_results).filter(([_, result]) => 
      selectedFilter === 'all' || result.detected
    ).sort((a, b) => {
      // Sort by detection status first, then by vendor name
      if (a[1].detected && !b[1].detected) return -1;
      if (!a[1].detected && b[1].detected) return 1;
      return a[0].localeCompare(b[0]);
    }) : [];

  function handleQuarantine() {
    dispatch('quarantine');
  }

  function handleReset() {
    dispatch('reset');
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
</script>

<div class="results-container">
  <div class="results-header">
    <h3>Scan Results</h3>
    <div class="scan-status {results.status.toLowerCase()}">
      <span class="status-icon">
        {#if results.status === 'Clean'}✅
        {:else if results.status === 'Suspicious'}⚠️
        {:else}🚫{/if}
      </span>
      <span class="status-text">{results.status}</span>
    </div>
  </div>

  <div class="file-info">
    <div class="info-grid">
      <div class="info-item">
        <span class="info-label">File Name</span>
        <span class="info-value">{results.file_name}</span>
      </div>
      <div class="info-item">
        <span class="info-label">Size</span>
        <span class="info-value">{formatBytes(results.file_size)}</span>
      </div>
      <div class="info-item">
        <span class="info-label">Scan Date</span>
        <span class="info-value">{formatDate(results.scan_date)}</span>
      </div>
      <div class="info-item">
        <span class="info-label">SHA-256</span>
        <span class="info-value">{results.file_hash}</span>
      </div>
    </div>
  </div>

  <div class="detection-summary">
    <div class="detection-rate {results.status.toLowerCase()}">
      <div class="rate-circle">
        <span class="rate-number">{results.detection_count || 0}</span>
        <span class="rate-total">/ {results.total_engines || 0}</span>
      </div>
      <div class="rate-label">Detections</div>
    </div>

    <div class="threat-info">
      <div class="threat-level {threatLevel.toLowerCase()}">
        <span class="level-label">Threat Level</span>
        <span class="level-value">{threatLevel}</span>
      </div>
      <div class="detection-bar">
        <div class="bar-track">
          <div 
            class="bar-fill" 
            style="width: {detectionRate}%"
          ></div>
        </div>
        <div class="bar-labels">
          <span>0%</span>
          <span>100%</span>
        </div>
      </div>
    </div>
  </div>

  {#if results.status !== 'Clean'}
    <div class="action-section">
      <h4>Recommended Actions</h4>
      <div class="action-buttons">
        <button class="quarantine-button" on:click={handleQuarantine}>
          <span class="button-icon">🔒</span>
          Move to Quarantine
        </button>
        <button class="delete-button" on:click={handleReset}>
          <span class="button-icon">🗑️</span>
          Delete File
        </button>
      </div>
    </div>
  {/if}

  <div class="vendor-results">
    <h4>Antivirus Results</h4>
    <div class="filter-buttons">
      <button 
        class="filter-button {selectedFilter === 'all' ? 'active' : ''}"
        on:click={() => selectedFilter = 'all'}
      >
        All Results
      </button>
      <button 
        class="filter-button {selectedFilter === 'detected' ? 'active' : ''}"
        on:click={() => selectedFilter = 'detected'}
      >
        Detections Only
      </button>
    </div>

    <div class="results-grid">
      {#each filteredResults as [vendor, result]}
        <div class="vendor-result {result.detected ? 'detected' : 'clean'}">
          <div class="vendor-info">
            <span class="vendor-name">{vendor}</span>
            {#if result.engine_version}
              <span class="vendor-version">{result.engine_version}</span>
            {/if}
          </div>
          <div class="result-info">
            {#if result.detected}
              <span class="detection-type">{result.result || 'Malicious'}</span>
            {:else}
              <span class="clean-result">Clean</span>
            {/if}
          </div>
        </div>
      {/each}
    </div>
  </div>

  {#if results.permalink}
    <div class="results-footer">
      <a 
        href={results.permalink} 
        target="_blank" 
        rel="noopener noreferrer"
        class="vt-link"
      >
        View Full Report on VirusTotal
      </a>
    </div>
  {/if}
</div>

<style>
  .results-container {
    background: var(--card-bg);
    border-radius: 12px;
    padding: 2rem;
    margin: 2rem 0;
    box-shadow: var(--shadow);
  }

  .results-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 2rem;
    padding-bottom: 1rem;
    border-bottom: 1px solid var(--border-color);
  }

  h3, h4 {
    margin: 0;
    color: var(--text-color);
  }

  .scan-status {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 1rem;
    border-radius: 20px;
    font-weight: 500;
  }

  .scan-status.clean {
    background: var(--success-subtle);
    color: var(--success);
  }

  .scan-status.suspicious {
    background: var(--warning-subtle);
    color: var(--warning);
  }

  .scan-status.malicious {
    background: var(--error-subtle);
    color: var(--error);
  }

  .info-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
    gap: 1.5rem;
    margin-bottom: 2rem;
  }

  .info-item {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .info-label {
    font-size: 0.875rem;
    color: var(--text-muted);
  }

  .info-value {
    font-weight: 500;
    word-break: break-all;
  }

  .detection-summary {
    display: flex;
    align-items: center;
    gap: 2rem;
    padding: 1.5rem;
    background: var(--bg-subtle);
    border-radius: 8px;
    margin-bottom: 2rem;
  }

  .detection-rate {
    text-align: center;
  }

  .rate-circle {
    width: 80px;
    height: 80px;
    border-radius: 50%;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    margin-bottom: 0.5rem;
  }

  .detection-rate.clean .rate-circle {
    background: var(--success-subtle);
    border: 2px solid var(--success);
  }

  .detection-rate.suspicious .rate-circle {
    background: var(--warning-subtle);
    border: 2px solid var(--warning);
  }

  .detection-rate.malicious .rate-circle {
    background: var(--error-subtle);
    border: 2px solid var(--error);
  }

  .rate-number {
    font-size: 1.5rem;
    font-weight: bold;
  }

  .rate-total {
    font-size: 0.875rem;
    color: var(--text-muted);
  }

  .rate-label {
    font-size: 0.875rem;
    color: var(--text-muted);
  }

  .threat-info {
    flex: 1;
  }

  .threat-level {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
  }

  .level-label {
    font-size: 0.875rem;
    color: var(--text-muted);
  }

  .level-value {
    font-weight: 500;
  }

  .threat-level.high .level-value {
    color: var(--error);
  }

  .threat-level.medium .level-value {
    color: var(--warning);
  }

  .threat-level.low .level-value {
    color: var(--warning-muted);
  }

  .threat-level.clean .level-value {
    color: var(--success);
  }

  .detection-bar {
    width: 100%;
  }

  .bar-track {
    height: 8px;
    background: var(--bg-muted);
    border-radius: 4px;
    overflow: hidden;
  }

  .bar-fill {
    height: 100%;
    background: var(--primary-color);
    transition: width 0.3s ease;
  }

  .bar-labels {
    display: flex;
    justify-content: space-between;
    font-size: 0.75rem;
    color: var(--text-muted);
    margin-top: 0.25rem;
  }

  .action-section {
    background: var(--error-subtle);
    padding: 1.5rem;
    border-radius: 8px;
    margin: 2rem 0;
  }

  .action-buttons {
    display: flex;
    gap: 1rem;
    margin-top: 1rem;
  }

  .quarantine-button,
  .delete-button {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    padding: 1rem;
    border: none;
    border-radius: 6px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
  }

  .quarantine-button {
    background: var(--warning);
    color: white;
  }

  .delete-button {
    background: var(--error);
    color: white;
  }

  .quarantine-button:hover,
  .delete-button:hover {
    transform: translateY(-2px);
    box-shadow: var(--shadow);
  }

  .vendor-results {
    margin-top: 2rem;
  }

  .filter-buttons {
    display: flex;
    gap: 0.5rem;
    margin: 1rem 0;
  }

  .filter-button {
    padding: 0.5rem 1rem;
    border: 1px solid var(--border-color);
    background: transparent;
    border-radius: 4px;
    color: var(--text-muted);
    cursor: pointer;
    transition: all 0.2s;
  }

  .filter-button.active {
    background: var(--primary-color);
    border-color: var(--primary-color);
    color: white;
  }

  .results-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: 1rem;
    margin-top: 1rem;
    max-height: 400px;
    overflow-y: auto;
    padding: 0.5rem;
  }

  .vendor-result {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem;
    border-radius: 6px;
    background: var(--bg-subtle);
  }

  .vendor-result.detected {
    background: var(--error-subtle);
  }

  .vendor-result.clean {
    background: var(--success-subtle);
  }

  .vendor-info {
    display: flex;
    flex-direction: column;
  }

  .vendor-name {
    font-weight: 500;
  }

  .vendor-version {
    font-size: 0.75rem;
    color: var(--text-muted);
  }

  .detection-type {
    color: var(--error);
    font-weight: 500;
  }

  .clean-result {
    color: var(--success);
    font-weight: 500;
  }

  .results-footer {
    margin-top: 2rem;
    padding-top: 1rem;
    border-top: 1px solid var(--border-color);
    text-align: center;
  }

  .vt-link {
    color: var(--primary-color);
    text-decoration: none;
    font-weight: 500;
  }

  .vt-link:hover {
    text-decoration: underline;
  }

  @media (max-width: 768px) {
    .detection-summary {
      flex-direction: column;
      align-items: stretch;
    }

    .action-buttons {
      flex-direction: column;
    }

    .results-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
