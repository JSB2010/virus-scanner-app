<!-- Settings.svelte -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke, open } from '../tauri';
  import type { Settings } from '../types';

  let settings: Settings = {
    api_key: '',
    welcome_completed: true,
    auto_scan_downloads: true,
    notify_on_scan_completion: true,
    auto_quarantine_malicious: false,
    auto_rescan_interval: 24,
    custom_scan_locations: [],
    scan_history_limit: 1000,
    file_type_filters: [],
    theme: 'system',
    startup_with_system: false,
    minimize_to_tray: true,
    background_scan_threads: 2
  };

  let isLoading = true;
  let error: string | null = null;
  let isSaving = false;
  let showApiKey = false;

  onMount(async () => {
    await loadSettings();
  });

  async function loadSettings() {
    try {
      isLoading = true;
      error = null;
      settings = await invoke('get_settings');
    } catch (err) {
      error = `Failed to load settings: ${err}`;
    } finally {
      isLoading = false;
    }
  }

  async function saveSettings() {
    try {
      isSaving = true;
      error = null;
      await invoke('update_settings', { settings });
      await invoke('show_notification', {
        title: 'Settings Saved',
        body: 'Your settings have been updated successfully.'
      });
    } catch (err) {
      error = `Failed to save settings: ${err}`;
    } finally {
      isSaving = false;
    }
  }

  async function testApiKey() {
    try {
      const isValid = await invoke('test_api_key', { apiKey: settings.api_key });
      if (isValid) {
        await invoke('show_notification', {
          title: 'API Key Valid',
          body: 'Your VirusTotal API key is valid and working.'
        });
      } else {
        error = 'Invalid API key. Please check your key and try again.';
      }
    } catch (err) {
      error = `Failed to test API key: ${err}`;
    }
  }

  async function addScanLocation() {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: 'Select Folder to Monitor'
      });

      if (selected && !Array.isArray(selected)) {
        if (!settings.custom_scan_locations.includes(selected)) {
          settings.custom_scan_locations = [...settings.custom_scan_locations, selected];
          await saveSettings();
          await invoke('add_monitoring_location', { path: selected });
        }
      }
    } catch (err) {
      error = `Failed to add monitoring location: ${err}`;
    }
  }

  async function removeScanLocation(path: string) {
    try {
      settings.custom_scan_locations = settings.custom_scan_locations.filter(loc => loc !== path);
      await saveSettings();
      await invoke('remove_monitoring_location', { path });
    } catch (err) {
      error = `Failed to remove monitoring location: ${err}`;
    }
  }

  async function exportData() {
    try {
      const filePath = await save({
        filters: [{
          name: 'JSON',
          extensions: ['json']
        }]
      });

      if (filePath) {
        const data = await invoke('export_data');
        await invoke('write_file', { path: filePath, content: data });
        await invoke('show_notification', {
          title: 'Export Complete',
          body: 'Settings and scan history have been exported successfully.'
        });
      }
    } catch (err) {
      error = `Failed to export data: ${err}`;
    }
  }

  async function importData() {
    try {
      const selected = await open({
        filters: [{
          name: 'JSON',
          extensions: ['json']
        }]
      });

      if (selected && !Array.isArray(selected)) {
        const content = await invoke('read_file', { path: selected });
        await invoke('import_data', { data: content });
        await loadSettings();
        await invoke('show_notification', {
          title: 'Import Complete',
          body: 'Settings and scan history have been imported successfully.'
        });
      }
    } catch (err) {
      error = `Failed to import data: ${err}`;
    }
  }

  async function resetToDefaults() {
    if (!confirm('Are you sure you want to reset all settings to their default values? This cannot be undone.')) {
      return;
    }

    try {
      await invoke('reset_settings');
      await loadSettings();
      await invoke('show_notification', {
        title: 'Settings Reset',
        body: 'All settings have been reset to their default values.'
      });
    } catch (err) {
      error = `Failed to reset settings: ${err}`;
    }
  }

  function validateSettings(): boolean {
    if (settings.scan_history_limit < 1) {
      error = 'Scan history limit must be at least 1';
      return false;
    }
    if (settings.background_scan_threads < 1) {
      error = 'Background scan threads must be at least 1';
      return false;
    }
    if (settings.auto_rescan_interval && settings.auto_rescan_interval < 1) {
      error = 'Auto-rescan interval must be at least 1 hour';
      return false;
    }
    return true;
  }

  $: if (settings) {
    const debounce = setTimeout(() => {
      if (validateSettings()) {
        saveSettings();
      }
    }, 500);
    return () => clearTimeout(debounce);
  }
</script>

<div class="settings-container">
  {#if error}
    <div class="error-message">
      {error}
      <button class="dismiss-button" on:click={() => error = null}>
        Dismiss
      </button>
    </div>
  {/if}

  {#if isLoading}
    <div class="loading">
      <div class="spinner"></div>
      <p>Loading settings...</p>
    </div>
  {:else}
    <div class="settings-grid">
      <!-- API Settings -->
      <section class="settings-section">
        <h3>API Configuration</h3>
        <div class="setting-group">
          <label for="api_key">VirusTotal API Key</label>
          <div class="api-key-input">
            <input
              type={showApiKey ? 'text' : 'password'}
              id="api_key"
              bind:value={settings.api_key}
            />
            <button class="icon-button" on:click={() => showApiKey = !showApiKey}>
              {showApiKey ? '👁️' : '👁️‍🗨️'}
            </button>
          </div>
          <button class="secondary-button" on:click={testApiKey}>
            Test API Key
          </button>
          <p class="help-text">
            Your VirusTotal API key is required for scanning files.
            <a
              href="https://www.virustotal.com/gui/join-us"
              target="_blank"
              rel="noopener noreferrer"
            >
              Get an API key
            </a>
          </p>
        </div>
      </section>

      <!-- Scanning Settings -->
      <section class="settings-section">
        <h3>Scanning Options</h3>
        <div class="setting-group">
          <label class="checkbox-label">
            <input
              type="checkbox"
              bind:checked={settings.auto_scan_downloads}
            />
            Automatically scan new downloads
          </label>
          <label class="checkbox-label">
            <input
              type="checkbox"
              bind:checked={settings.notify_on_scan_completion}
            />
            Show notifications when scans complete
          </label>
          <label class="checkbox-label">
            <input
              type="checkbox"
              bind:checked={settings.auto_quarantine_malicious}
            />
            Automatically quarantine malicious files
          </label>
        </div>

        <div class="setting-group">
          <label for="auto_rescan">Auto-Rescan Interval (hours)</label>
          <input
            type="number"
            id="auto_rescan"
            bind:value={settings.auto_rescan_interval}
            min="1"
            step="1"
          />
          <p class="help-text">
            How often quarantined files should be automatically rescanned. Set to 0 to disable.
          </p>
        </div>

        <div class="setting-group">
          <label for="scan_threads">Background Scan Threads</label>
          <input
            type="number"
            id="scan_threads"
            bind:value={settings.background_scan_threads}
            min="1"
            max="16"
            step="1"
          />
          <p class="help-text">
            Number of concurrent scans to run in the background (1-16)
          </p>
        </div>
      </section>

      <!-- Monitored Locations -->
      <section class="settings-section">
        <h3>Monitored Locations</h3>
        <div class="setting-group">
          <button class="primary-button" on:click={addScanLocation}>
            Add Folder to Monitor
          </button>

          <div class="locations-list">
            {#each settings.custom_scan_locations as location}
              <div class="location-item">
                <span class="location-path">{location}</span>
                <button
                  class="remove-button"
                  on:click={() => removeScanLocation(location)}
                >
                  Remove
                </button>
              </div>
            {/each}
          </div>

          <p class="help-text">
            Add folders to monitor for new files in addition to the Downloads folder
          </p>
        </div>
      </section>

      <!-- Application Settings -->
      <section class="settings-section">
        <h3>Application Settings</h3>
        <div class="setting-group">
          <label for="theme">Theme</label>
          <select id="theme" bind:value={settings.theme}>
            <option value="system">System Default</option>
            <option value="light">Light</option>
            <option value="dark">Dark</option>
          </select>
        </div>

        <div class="setting-group">
          <label class="checkbox-label">
            <input
              type="checkbox"
              bind:checked={settings.startup_with_system}
            />
            Start with system
          </label>
          <label class="checkbox-label">
            <input
              type="checkbox"
              bind:checked={settings.minimize_to_tray}
            />
            Minimize to system tray
          </label>
        </div>

        <div class="setting-group">
          <label for="history_limit">Scan History Limit</label>
          <input
            type="number"
            id="history_limit"
            bind:value={settings.scan_history_limit}
            min="1"
            step="100"
          />
          <p class="help-text">
            Maximum number of scan results to keep in history
          </p>
        </div>
      </section>

      <!-- Data Management -->
      <section class="settings-section">
        <h3>Data Management</h3>
        <div class="setting-group button-group">
          <button class="secondary-button" on:click={exportData}>
            Export Settings & History
          </button>
          <button class="secondary-button" on:click={importData}>
            Import Settings & History
          </button>
          <button class="danger-button" on:click={resetToDefaults}>
            Reset to Defaults
          </button>
        </div>
      </section>
    </div>
  {/if}
</div>

<style>
  .settings-container {
    max-width: 800px;
    margin: 0 auto;
    padding: 2rem;
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

  .settings-grid {
    display: grid;
    gap: 2rem;
  }

  .settings-section {
    background: var(--card-bg);
    border-radius: 12px;
    padding: 1.5rem;
    box-shadow: var(--shadow);
  }

  h3 {
    margin: 0 0 1.5rem 0;
    color: var(--text-color);
    font-size: 1.25rem;
  }

  .setting-group {
    margin-bottom: 1.5rem;
  }

  .setting-group:last-child {
    margin-bottom: 0;
  }

  label {
    display: block;
    margin-bottom: 0.5rem;
    color: var(--text-color);
  }

  .checkbox-label {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin: 0.5rem 0;
    cursor: pointer;
  }

  input[type="text"],
  input[type="password"],
  input[type="number"],
  select {
    width: 100%;
    padding: 0.75rem;
    border: 1px solid var(--border-color);
    border-radius: 6px;
    background: var(--bg-subtle);
    color: var(--text-color);
    margin-bottom: 0.5rem;
  }

  input[type="checkbox"] {
    width: 1.2rem;
    height: 1.2rem;
    cursor: pointer;
  }

  .api-key-input {
    display: flex;
    gap: 0.5rem;
  }

  .icon-button {
    background: none;
    border: none;
    cursor: pointer;
    padding: 0.75rem;
    border-radius: 6px;
  }

  .icon-button:hover {
    background: var(--bg-subtle);
  }

  .help-text {
    font-size: 0.875rem;
    color: var(--text-muted);
    margin: 0.5rem 0;
  }

  .help-text a {
    color: var(--primary-color);
    text-decoration: none;
  }

  .help-text a:hover {
    text-decoration: underline;
  }

  .button-group {
    display: flex;
    gap: 1rem;
    flex-wrap: wrap;
  }

  .primary-button,
  .secondary-button,
  .danger-button {
    padding: 0.75rem 1.5rem;
    border: none;
    border-radius: 6px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
  }

  .primary-button {
    background: var(--primary-color);
    color: white;
  }

  .secondary-button {
    background: var(--secondary-color);
    color: white;
  }

  .danger-button {
    background: var(--error);
    color: white;
  }

  .primary-button:hover,
  .secondary-button:hover,
  .danger-button:hover {
    transform: translateY(-2px);
    box-shadow: var(--shadow);
  }

  .locations-list {
    margin: 1rem 0;
  }

  .location-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.75rem 1rem;
    background: var(--bg-subtle);
    border-radius: 6px;
    margin-bottom: 0.5rem;
  }

  .location-path {
    word-break: break-all;
    font-size: 0.875rem;
  }

  .remove-button {
    padding: 0.25rem 0.75rem;
    background: var(--error);
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
  }

  .remove-button:hover {
    opacity: 0.9;
  }

  @media (max-width: 640px) {
    .settings-container {
      padding: 1rem;
    }

    .button-group {
      flex-direction: column;
    }

    .primary-button,
    .secondary-button,
    .danger-button {
      width: 100%;
    }
  }
</style>
