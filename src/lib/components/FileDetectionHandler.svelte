<!-- FileDetectionHandler.svelte -->
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { listen, invoke } from '../tauri.js';
  import ScanPrompt from './ScanPrompt.svelte';

  // Props
  export let onScanComplete = (result: any) => {};
  export let onTabChange = (tab: string) => {};

  // State
  let showPrompt = false;
  let detectedFile = {
    fileName: '',
    filePath: ''
  };
  let unlistenFunction: () => void;

  // Setup event listener for file detection
  onMount(async () => {
    unlistenFunction = await listen('file-detected', (event: any) => {
      const payload = event.payload as { file_name: string, path: string };
      detectedFile = {
        fileName: payload.file_name,
        filePath: payload.path
      };
      showPrompt = true;
      console.log('File detected:', detectedFile);
    });
  });

  // Clean up event listener
  onDestroy(() => {
    if (unlistenFunction) {
      unlistenFunction();
    }
  });

  // Handle scan action
  async function handleScan(event: CustomEvent) {
    showPrompt = false;
    const { filePath } = event.detail as { filePath: string };

    try {
      const result = await invoke('scan_file', { filePath });
      onScanComplete(result);
      onTabChange('scanner');
    } catch (err) {
      console.error('Error scanning file:', err);
    }
  }

  // Handle ignore action
  function handleIgnore() {
    showPrompt = false;
  }
</script>

{#if showPrompt}
  <ScanPrompt
    fileName={detectedFile.fileName}
    filePath={detectedFile.filePath}
    on:scan={handleScan}
    on:ignore={handleIgnore}
  />
{/if}
