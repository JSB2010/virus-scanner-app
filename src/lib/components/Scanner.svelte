<!-- Scanner.svelte -->
<script lang="ts">
  import { invoke, open, listen } from '../tauri.js';
  import type { ScanResult } from '../types';
  import FileInfo from './FileInfo.svelte';
  import ScanProgress from './ScanProgress.svelte';
  import ScanResults from './ScanResults.svelte';

  // Props
  export let settings = {
    auto_scan_downloads: true,
    notify_on_scan_completion: true,
    auto_quarantine_malicious: false
  };

  // State
  let currentFile: string | null = null;
  let isScanning = false;
  let scanStep = 1;
  let scanStatusMessage = '';
  let scanResults: ScanResult | null = null;
  let error: string | null = null;
  let isDragging = false;

  // Handle file selection
  async function selectFile() {
    try {
      const selected = await open({
        multiple: false,
        filters: [{
          name: 'All Files',
          extensions: ['*']
        }]
      });

      if (selected && !Array.isArray(selected)) {
        currentFile = selected;
        await startScan();
      }
    } catch (err) {
      error = `Error selecting file: ${err}`;
    }
  }

  // Start scanning process
  async function startScan() {
    if (!currentFile) {
      error = 'No file selected';
      return;
    }

    try {
      error = null;
      isScanning = true;
      scanStep = 1;
      scanStatusMessage = 'Preparing file for scanning...';

      // Listen for scan progress updates
      const unlistenProgress = await listen('scan_progress', (event: any) => {
        const { step, message, progress } = event.payload as { step: number, message: string, progress: number };
        scanStep = step;
        scanStatusMessage = message;
        console.log(`Scan progress: ${message} (${progress}%)`);
      });

      // Start the scan
      const result = await invoke('scan_file', { filePath: currentFile });
      scanResults = result as ScanResult;

      // Remove progress listener
      unlistenProgress();

      if (scanResults && scanResults.status === 'Malicious' && settings.auto_quarantine_malicious) {
        await quarantineFile();
      }

      if (scanResults && settings.notify_on_scan_completion) {
        const status = scanResults.status.toLowerCase();
        const message = `Scan complete: File is ${status}`;
        await invoke('show_notification', {
          title: 'Scan Complete',
          body: message
        });
      }
    } catch (err) {
      error = `Error scanning file: ${err}`;
      scanResults = null;
    } finally {
      isScanning = false;
    }
  }

  // Handle drag and drop
  function handleDragEnter(event: DragEvent) {
    event.preventDefault();
    isDragging = true;
  }

  function handleDragLeave(event: DragEvent) {
    event.preventDefault();
    isDragging = false;
  }

  function handleDragOver(event: DragEvent) {
    event.preventDefault();
  }

  async function handleDrop(event: DragEvent) {
    event.preventDefault();
    isDragging = false;

    const files = event.dataTransfer?.files;
    if (files && files.length > 0) {
      // We need to get the file path from the dropped file
      // In a real app, we would use the Tauri API to get the file path
      // For now, we'll just use a workaround
      const filePath = await invoke('get_dropped_file_path', { fileName: files[0].name });
      if (filePath) {
        currentFile = filePath as string;
        await startScan();
      }
    }
  }

  // Handle file quarantine
  async function quarantineFile() {
    if (!scanResults) return;

    try {
      await invoke('quarantine_file', { filePath: currentFile });
      const message = `File ${currentFile} has been quarantined`;
      await invoke('show_notification', {
        title: 'File Quarantined',
        body: message
      });
    } catch (err) {
      error = `Error quarantining file: ${err}`;
    }
  }

  // Reset scan state
  function resetScan() {
    currentFile = null;
    scanResults = null;
    error = null;
    isScanning = false;
    scanStep = 1;
    scanStatusMessage = '';
  }
</script>

<div class="scanner-container">
  <div class="control-panel">
    <button class="scan-button" on:click={selectFile} disabled={isScanning}>
      Select File to Scan
    </button>
  </div>

  <div
    class="drop-area {isDragging ? 'active' : ''}"
    on:dragenter={handleDragEnter}
    on:dragleave={handleDragLeave}
    on:dragover={handleDragOver}
    on:drop={handleDrop}
    role="region"
    aria-label="Drop zone for files"
  >
    <div class="drop-content">
      <div class="drop-icon">
        {#if isDragging}📂{:else}📄{/if}
      </div>
      <p class="drop-text">
        {#if isDragging}
          Drop file to scan
        {:else}
          Drag and drop a file here to scan
        {/if}
      </p>
      <p class="drop-subtext">or use the button above</p>
    </div>
  </div>

  {#if error}
    <div class="error-message">
      {error}
    </div>
  {/if}

  {#if isScanning}
    <ScanProgress {scanStep} {scanStatusMessage} />
  {/if}

  {#if currentFile && !isScanning}
    <FileInfo filePath={currentFile} />
  {/if}

  {#if scanResults}
    <ScanResults
      results={scanResults}
      on:quarantine={quarantineFile}
      on:reset={resetScan}
      {settings}
    />
  {/if}
</div>

<style>
  .scanner-container {
    max-width: 800px;
    margin: 0 auto;
    padding: 2rem;
  }

  .control-panel {
    text-align: center;
    margin-bottom: 2rem;
  }

  .scan-button {
    padding: 1rem 2rem;
    background: var(--primary-color);
    color: white;
    border: none;
    border-radius: 8px;
    font-size: 1.1rem;
    cursor: pointer;
    transition: all 0.2s;
  }

  .scan-button:hover {
    transform: translateY(-2px);
    box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
  }

  .scan-button:disabled {
    opacity: 0.7;
    cursor: not-allowed;
    transform: none;
  }

  .drop-area {
    margin: 2rem 0;
    padding: 3rem;
    border: 2px dashed var(--border-color);
    border-radius: 12px;
    background: var(--bg-subtle);
    text-align: center;
    transition: all 0.3s;
    cursor: pointer;
  }

  .drop-area.active {
    border-color: var(--primary-color);
    background: var(--primary-subtle);
    transform: scale(1.02);
  }

  .drop-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
  }

  .drop-icon {
    font-size: 3rem;
    margin-bottom: 1rem;
    color: var(--text-muted);
  }

  .drop-text {
    font-size: 1.25rem;
    font-weight: 500;
    margin-bottom: 0.5rem;
    color: var(--text-color);
  }

  .drop-subtext {
    font-size: 0.875rem;
    color: var(--text-muted);
  }

  .error-message {
    margin: 1rem 0;
    padding: 1rem;
    background: var(--error-subtle);
    color: var(--error);
    border-radius: 8px;
    border: 1px solid var(--error-muted);
  }
</style>
