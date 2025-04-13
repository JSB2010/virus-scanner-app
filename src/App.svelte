<!-- App.svelte -->
<script>
  import { onMount } from 'svelte';
  import { invoke } from './lib/tauri';

  let loading = true;
  let error = null;
  let settings = null;
  let activeTab = 'welcome';
  let apiKey = '';

  onMount(async () => {
    try {
      loading = true;
      // Try to load settings
      settings = await invoke('get_settings').catch(() => ({
        api_key: '',
        welcome_completed: false,
        auto_scan_downloads: true,
        notify_on_scan_completion: true,
        auto_quarantine_malicious: false,
        scan_history_limit: 100,
        theme: 'system',
        startup_with_system: false,
        minimize_to_tray: true,
      }));

      apiKey = settings.api_key;
      activeTab = settings.welcome_completed ? 'scanner' : 'welcome';
      loading = false;
    } catch (e) {
      console.error('Failed to initialize app:', e);
      error = e.toString();
      loading = false;
    }
  });

  async function saveSettings() {
    try {
      loading = true;
      settings.api_key = apiKey;
      settings.welcome_completed = true;
      await invoke('update_settings', { settings });
      activeTab = 'scanner';
      loading = false;
    } catch (e) {
      console.error('Failed to save settings:', e);
      error = e.toString();
      loading = false;
    }
  }
</script>

<main>
  {#if loading}
    <div class="loading">
      <div class="spinner"></div>
      <p>Loading...</p>
    </div>
  {:else if error}
    <div class="error">
      <h2>Error</h2>
      <p>{error}</p>
      <button on:click={() => window.location.reload()}>Retry</button>
    </div>
  {:else if activeTab === 'welcome'}
    <div class="welcome">
      <h1>Welcome to VirusTotal Scanner</h1>
      <p>This application helps you scan files for viruses using VirusTotal's powerful scanning engine.</p>

      <div class="setup-form">
        <h2>Setup</h2>
        <p>Please enter your VirusTotal API key to get started:</p>

        <div class="form-group">
          <label for="api-key">API Key:</label>
          <input
            type="password"
            id="api-key"
            bind:value={apiKey}
            placeholder="Enter your VirusTotal API key"
          />
          <p class="help-text">Don't have an API key? <a href="https://www.virustotal.com/gui/join-us" target="_blank">Sign up for free at VirusTotal</a></p>
        </div>

        <button class="primary" on:click={saveSettings} disabled={!apiKey}>Complete Setup</button>
      </div>
    </div>
  {:else}
    <div class="scanner">
      <h1>VirusTotal Scanner</h1>
      <p>The application is now ready to use. You can scan files, view history, and manage settings.</p>

      <div class="tabs">
        <button class="tab {activeTab === 'scanner' ? 'active' : ''}" on:click={() => activeTab = 'scanner'}>Scanner</button>
        <button class="tab {activeTab === 'history' ? 'active' : ''}" on:click={() => activeTab = 'history'}>History</button>
        <button class="tab {activeTab === 'settings' ? 'active' : ''}" on:click={() => activeTab = 'settings'}>Settings</button>
      </div>

      <div class="tab-content">
        {#if activeTab === 'scanner'}
          <div class="scanner-content">
            <h2>File Scanner</h2>
            <p>Select a file to scan for viruses.</p>
            <button class="primary">Select File</button>
          </div>
        {:else if activeTab === 'history'}
          <div class="history-content">
            <h2>Scan History</h2>
            <p>Your scan history will appear here.</p>
          </div>
        {:else if activeTab === 'settings'}
          <div class="settings-content">
            <h2>Settings</h2>
            <div class="form-group">
              <label for="settings-api-key">API Key:</label>
              <input
                type="password"
                id="settings-api-key"
                bind:value={apiKey}
                placeholder="Enter your VirusTotal API key"
              />
            </div>
            <button class="primary" on:click={saveSettings}>Save Settings</button>
          </div>
        {/if}
      </div>
    </div>
  {/if}
</main>

<style>
  main {
    max-width: 1200px;
    margin: 0 auto;
    padding: 2rem;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, 'Open Sans', 'Helvetica Neue', sans-serif;
  }

  h1 {
    font-size: 2rem;
    margin-bottom: 1rem;
    color: #3182ce;
  }

  h2 {
    font-size: 1.5rem;
    margin-bottom: 1rem;
  }

  p {
    margin-bottom: 1rem;
    line-height: 1.5;
  }

  .loading {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100vh;
  }

  .spinner {
    width: 40px;
    height: 40px;
    border: 4px solid rgba(0, 0, 0, 0.1);
    border-radius: 50%;
    border-top-color: #3182ce;
    animation: spin 1s ease-in-out infinite;
    margin-bottom: 1rem;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .error {
    max-width: 500px;
    margin: 0 auto;
    padding: 2rem;
    background-color: #fff5f5;
    border: 1px solid #feb2b2;
    border-radius: 0.5rem;
    text-align: center;
  }

  .error h2 {
    color: #e53e3e;
  }

  .welcome, .scanner {
    max-width: 800px;
    margin: 0 auto;
  }

  .setup-form {
    background-color: #fff;
    border-radius: 0.5rem;
    padding: 2rem;
    box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
    margin-top: 2rem;
  }

  .form-group {
    margin-bottom: 1.5rem;
  }

  label {
    display: block;
    margin-bottom: 0.5rem;
    font-weight: 500;
  }

  input {
    width: 100%;
    padding: 0.75rem;
    border: 1px solid #e2e8f0;
    border-radius: 0.25rem;
    font-size: 1rem;
  }

  .help-text {
    font-size: 0.875rem;
    color: #718096;
    margin-top: 0.5rem;
  }

  button {
    padding: 0.75rem 1.5rem;
    border: none;
    border-radius: 0.25rem;
    font-size: 1rem;
    cursor: pointer;
    transition: background-color 0.2s;
  }

  button.primary {
    background-color: #3182ce;
    color: white;
  }

  button.primary:hover {
    background-color: #2c5282;
  }

  button.primary:disabled {
    background-color: #a0aec0;
    cursor: not-allowed;
  }

  .tabs {
    display: flex;
    border-bottom: 1px solid #e2e8f0;
    margin-bottom: 1.5rem;
  }

  .tab {
    padding: 0.75rem 1.5rem;
    background: none;
    border: none;
    border-bottom: 2px solid transparent;
    color: #718096;
    cursor: pointer;
  }

  .tab:hover {
    color: #4a5568;
  }

  .tab.active {
    color: #3182ce;
    border-bottom-color: #3182ce;
  }

  .tab-content {
    padding: 1.5rem 0;
  }

  a {
    color: #3182ce;
    text-decoration: none;
  }

  a:hover {
    text-decoration: underline;
  }

  :global(:root) {
    /* Light theme variables */
    --bg-color: #f8f9fa;
    --card-bg: #ffffff;
    --text-color: #2d3748;
    --text-muted: #718096;
    --border-color: #e2e8f0;
    --primary-color: #3182ce;
    --primary-subtle: #ebf8ff;
    --secondary-color: #4a5568;
    --success: #38a169;
    --success-subtle: #f0fff4;
    --warning: #d69e2e;
    --warning-subtle: #fffff0;
    --error: #e53e3e;
    --error-subtle: #fff5f5;
    --error-muted: #feb2b2;
    --bg-subtle: #f7fafc;
    --bg-muted: #edf2f7;
    --shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
    --shadow-lg: 0 10px 15px -3px rgba(0, 0, 0, 0.1);
  }

  :global([data-theme="dark"]) {
    /* Dark theme variables */
    --bg-color: #1a202c;
    --card-bg: #2d3748;
    --text-color: #f7fafc;
    --text-muted: #a0aec0;
    --border-color: #4a5568;
    --primary-color: #4299e1;
    --primary-subtle: #2a4365;
    --secondary-color: #a0aec0;
    --success: #48bb78;
    --success-subtle: #1c4532;
    --warning: #ecc94b;
    --warning-subtle: #744210;
    --error: #f56565;
    --error-subtle: #742a2a;
    --error-muted: #9b2c2c;
    --bg-subtle: #2d3748;
    --bg-muted: #4a5568;
    --shadow: 0 2px 4px rgba(0, 0, 0, 0.25);
    --shadow-lg: 0 10px 15px -3px rgba(0, 0, 0, 0.25);
  }

  :global(html, body) {
    margin: 0;
    padding: 0;
    font-family: system-ui, -apple-system, sans-serif;
  }

  :global(*, *::before, *::after) {
    box-sizing: border-box;
  }

  :global(body) {
    background: var(--bg-color);
    color: var(--text-color);
    line-height: 1.5;
    font-size: 16px;
  }

  :global(button) {
    font-family: inherit;
  }

  :global(::-webkit-scrollbar) {
    width: 8px;
    height: 8px;
  }

  :global(::-webkit-scrollbar-track) {
    background: var(--bg-subtle);
  }

  :global(::-webkit-scrollbar-thumb) {
    background: var(--border-color);
    border-radius: 4px;
  }

  :global(::-webkit-scrollbar-thumb:hover) {
    background: var(--text-muted);
  }
</style>
