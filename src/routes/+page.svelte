<script>
  import { onMount, onDestroy } from 'svelte';
  import { invoke, listen, open, isPermissionGranted, requestPermission, sendNotification } from '../lib/tauri.js';

  // Define data as a constant since we're not using it
  export const data = { ready: true };

  // App state
  let darkMode = false;
  let loading = false;
  let activeTab = 'scanner';
  let activeWelcomeStep = 0;
  let showNotification = false;
  let notificationMessage = '';
  let notificationType = 'info';

  // User settings
  let settings = {
    api_key: '',
    welcome_completed: false,
    auto_scan_downloads: true,
    notify_on_scan_completion: true,
    auto_quarantine_malicious: false,
    scan_history_limit: 100,
    theme: 'system',
    startup_with_system: false,
    minimize_to_tray: true,
  };

  // Scan data
  let scanHistory = [];
  let quarantinedFiles = [];
  let activeScans = [];
  let isMonitoring = false;

  // Event unlisten functions
  let unlistenFunctions = [];

  // Toggle dark mode
  function toggleDarkMode() {
    if (settings.theme === 'system') {
      // If theme is set to system, switch to explicit light/dark
      settings.theme = darkMode ? 'light' : 'dark';
    } else {
      // Toggle between light and dark
      settings.theme = settings.theme === 'dark' ? 'light' : 'dark';
    }

    updateTheme();
    saveSettings();
  }

  // Update theme based on settings
  function updateTheme() {
    if (settings.theme === 'system') {
      const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
      darkMode = prefersDark;
    } else {
      darkMode = settings.theme === 'dark';
    }

    document.body.classList.toggle('dark-mode', darkMode);
  }

  // Show notification
  async function showNotificationMessage(message, type = 'info') {
    // Show in-app notification
    notificationMessage = message;
    notificationType = type;
    showNotification = true;

    // Auto-hide after 5 seconds
    setTimeout(() => {
      showNotification = false;
    }, 5000);

    // Also send a system notification if enabled
    if (settings.notify_on_scan_completion) {
      try {
        // Check if we have permission
        let permissionGranted = await isPermissionGranted();
        if (!permissionGranted) {
          const permission = await requestPermission();
          permissionGranted = permission === 'granted';
        }

        if (permissionGranted) {
          await sendNotification({
            title: type.charAt(0).toUpperCase() + type.slice(1),
            body: message
          });
        }
      } catch (e) {
        console.error('Failed to send system notification:', e);
      }
    }
  }

  // Load settings from backend
  async function loadSettings() {
    loading = true;
    try {
      settings = await invoke('get_settings');
      updateTheme();
      loading = false;
    } catch (e) {
      console.error('Error loading settings:', e);
      showNotificationMessage(`Failed to load settings: ${e}`, 'error');
      loading = false;
    }
  }

  // Save settings to backend
  async function saveSettings() {
    loading = true;
    try {
      await invoke('update_settings', { settings });
      showNotificationMessage('Settings saved successfully', 'success');
      loading = false;
    } catch (e) {
      console.error('Error saving settings:', e);
      showNotificationMessage(`Failed to save settings: ${e}`, 'error');
      loading = false;
    }
  }

  // Complete welcome setup
  async function completeWelcome() {
    if (!settings.api_key) {
      showNotificationMessage('Please enter a VirusTotal API key', 'error');
      return;
    }

    loading = true;
    try {
      settings.welcome_completed = true;
      await saveSettings();
      loading = false;
    } catch (e) {
      console.error('Error completing welcome:', e);
      loading = false;
    }
  }

  // Load scan history
  async function loadScanHistory() {
    try {
      scanHistory = await invoke('get_scan_history');
    } catch (e) {
      console.error('Error loading scan history:', e);
      showNotificationMessage(`Failed to load scan history: ${e}`, 'error');
    }
  }

  // Load quarantined files
  async function loadQuarantinedFiles() {
    try {
      quarantinedFiles = await invoke('get_quarantined_files');
    } catch (e) {
      console.error('Error loading quarantined files:', e);
      showNotificationMessage(`Failed to load quarantined files: ${e}`, 'error');
    }
  }

  // Start file monitoring
  async function startMonitoring() {
    try {
      await invoke('start_monitoring');
      isMonitoring = true;
      showNotificationMessage('Started monitoring downloads folder', 'success');
    } catch (e) {
      console.error('Error starting monitoring:', e);
      showNotificationMessage(`Failed to start monitoring: ${e}`, 'error');
    }
  }

  // Stop file monitoring
  async function stopMonitoring() {
    try {
      await invoke('stop_monitoring');
      isMonitoring = false;
      showNotificationMessage('Stopped monitoring downloads folder', 'info');
    } catch (e) {
      console.error('Error stopping monitoring:', e);
      showNotificationMessage(`Failed to stop monitoring: ${e}`, 'error');
    }
  }

  // Scan a file
  async function scanFile(filePath) {
    loading = true;
    try {
      const result = await invoke('scan_file', { filePath });
      activeScans = [...activeScans, result];
      showNotificationMessage(`Started scanning ${result.file_name}`, 'info');
      loading = false;

      // Refresh scan history after a delay to allow the scan to complete
      setTimeout(loadScanHistory, 5000);
      return result;
    } catch (e) {
      console.error('Error scanning file:', e);
      showNotificationMessage(`Failed to scan file: ${e}`, 'error');
      loading = false;
      return null;
    }
  }

  // Open file dialog and scan selected file
  async function openFileDialog() {
    try {
      const selected = await open({
        multiple: false,
        filters: [{
          name: 'All Files',
          extensions: ['*']
        }]
      });

      if (selected) {
        await scanFile(selected);
        activeTab = 'scanner';
      }
    } catch (e) {
      console.error('Error opening file dialog:', e);
      showNotificationMessage(`Failed to open file dialog: ${e}`, 'error');
    }

    return null;
  }

  // Scan downloads folder
  async function scanDownloadsFolder() {
    loading = true;
    try {
      // Use the Rust backend to scan the downloads folder
      await invoke('scan_downloads_folder');
      showNotificationMessage(`Scanning files from downloads folder`, 'info');

      // Refresh scan history after a delay
      setTimeout(loadScanHistory, 5000);
      loading = false;
    } catch (e) {
      console.error('Error scanning downloads folder:', e);
      showNotificationMessage(`Failed to scan downloads folder: ${e}`, 'error');
      loading = false;
    }
  }

  // Restore a quarantined file
  async function restoreFile(quarantineId) {
    loading = true;
    try {
      await invoke('restore_file', { quarantineId });
      showNotificationMessage('File restored successfully', 'success');
      loadQuarantinedFiles();
      loading = false;
    } catch (e) {
      console.error('Error restoring file:', e);
      showNotificationMessage(`Failed to restore file: ${e}`, 'error');
      loading = false;
    }
  }

  // Delete a quarantined file
  async function deleteFile(quarantineId) {
    loading = true;
    try {
      await invoke('delete_file', { quarantineId });
      showNotificationMessage('File deleted successfully', 'success');
      loadQuarantinedFiles();
      loading = false;
    } catch (e) {
      console.error('Error deleting file:', e);
      showNotificationMessage(`Failed to delete file: ${e}`, 'error');
      loading = false;
    }
  }

  // Setup event listeners
  async function setupEventListeners() {
    // Listen for scan-downloads event from system tray
    const unlisten1 = await listen('scan-downloads', () => {
      scanDownloadsFolder();
      activeTab = 'scanner';
    });

    // Listen for open-settings event from system tray
    const unlisten2 = await listen('open-settings', () => {
      activeTab = 'settings';
    });

    // Listen for open-file-dialog event from system tray
    const unlisten3 = await listen('open-file-dialog', () => {
      openFileDialog();
      activeTab = 'scanner';
    });

    // Listen for scan-complete event
    const unlisten4 = await listen('scan-complete', (event) => {
      const { payload } = event;
      loadScanHistory();

      // Update active scans
      activeScans = activeScans.filter(scan => scan.id !== payload.id);

      // Show notification
      const statusText = payload.status === 'Clean' ? 'clean' :
                        payload.status === 'Suspicious' ? 'suspicious' :
                        payload.status === 'Malicious' ? 'malicious' : 'unknown';

      showNotificationMessage(`Scan complete: ${payload.file_name} is ${statusText}`,
                             statusText === 'clean' ? 'success' :
                             statusText === 'suspicious' ? 'warning' : 'error');
    });

    // Listen for new-file-detected event
    const unlisten5 = await listen('new-file-detected', async (event) => {
      const filePath = event.payload;
      const fileName = filePath.split(/[\\/]/).pop() || 'Unknown file';

      // Show notification with confirmation
      if (confirm(`New file detected: ${fileName}\nWould you like to scan it?`)) {
        await scanFile(filePath);
        activeTab = 'scanner';
      }
    });

    // Add to unlistenFunctions array
    unlistenFunctions.push(unlisten1, unlisten2, unlisten3, unlisten4, unlisten5);
  }

  // Initialize the application
  onMount(async () => {
    // Load settings
    await loadSettings();

    // If welcome is completed, load data
    if (settings.welcome_completed) {
      await Promise.all([
        loadScanHistory(),
        loadQuarantinedFiles()
      ]);

      // Start monitoring if enabled
      if (settings.auto_scan_downloads) {
        startMonitoring();
      }
    }

    // Setup event listeners
    await setupEventListeners();
  });

  // Cleanup on component destroy
  onDestroy(() => {
    // Remove event listeners
    unlistenFunctions.forEach(unlisten => unlisten());

    // Stop monitoring if active
    if (isMonitoring) {
      stopMonitoring();
    }
  });
</script>

<main class={darkMode ? 'dark' : 'light'}>
  <div class="container">
    <header>
      <h1>VirusTotal Scanner</h1>

      {#if settings.welcome_completed}
        <nav>
          <ul>
            <li class={activeTab === 'scanner' ? 'active' : ''}>
              <button on:click={() => activeTab = 'scanner'}>Scanner</button>
            </li>
            <li class={activeTab === 'history' ? 'active' : ''}>
              <button on:click={() => activeTab = 'history'}>History</button>
            </li>
            <li class={activeTab === 'quarantine' ? 'active' : ''}>
              <button on:click={() => activeTab = 'quarantine'}>Quarantine</button>
            </li>
            <li class={activeTab === 'settings' ? 'active' : ''}>
              <button on:click={() => activeTab = 'settings'}>Settings</button>
            </li>
            <li>
              <button class="icon-button" on:click={toggleDarkMode}>
                {#if darkMode}
                  <span>🌞</span>
                {:else}
                  <span>🌙</span>
                {/if}
              </button>
            </li>
          </ul>
        </nav>
      {/if}
    </header>

    <div class="content">
      {#if !settings.welcome_completed}
        <!-- Welcome Screen -->
        <div class="card welcome-card">
          <div class="welcome-steps">
            <div class="step-indicators">
              <div class="step-indicator {activeWelcomeStep === 0 ? 'active' : ''}"></div>
              <div class="step-indicator {activeWelcomeStep === 1 ? 'active' : ''}"></div>
              <div class="step-indicator {activeWelcomeStep === 2 ? 'active' : ''}"></div>
            </div>

            {#if activeWelcomeStep === 0}
              <h2>Welcome to VirusTotal Scanner</h2>
              <p>This application helps keep your system secure by automatically scanning new downloads with VirusTotal's extensive virus detection engines.</p>

              <div class="welcome-content">
                <div class="welcome-image">
                  <span class="emoji-icon">🛡️</span>
                </div>
                <div class="welcome-text">
                  <h3>Key Features:</h3>
                  <ul>
                    <li>Automatic monitoring of your Downloads folder</li>
                    <li>Scan files with 70+ antivirus engines</li>
                    <li>Detailed scan results and threat information</li>
                    <li>Quarantine management for suspicious files</li>
                    <li>System tray integration for easy access</li>
                  </ul>
                </div>
              </div>

              <div class="actions">
                <button class="primary" on:click={() => activeWelcomeStep = 1}>Next</button>
              </div>
            {:else if activeWelcomeStep === 1}
              <h2>How It Works</h2>
              <p>VirusTotal Scanner runs in the background and helps you stay protected from malicious files.</p>

              <div class="welcome-content">
                <div class="how-it-works">
                  <div class="step">
                    <span class="step-number">1</span>
                    <div class="step-content">
                      <h4>Detection</h4>
                      <p>When a new file appears in your Downloads folder, you'll get a notification</p>
                    </div>
                  </div>

                  <div class="step">
                    <span class="step-number">2</span>
                    <div class="step-content">
                      <h4>Analysis</h4>
                      <p>The file is securely uploaded to VirusTotal and analyzed by multiple antivirus engines</p>
                    </div>
                  </div>

                  <div class="step">
                    <span class="step-number">3</span>
                    <div class="step-content">
                      <h4>Results</h4>
                      <p>View detailed scan results showing which engines detected threats</p>
                    </div>
                  </div>

                  <div class="step">
                    <span class="step-number">4</span>
                    <div class="step-content">
                      <h4>Action</h4>
                      <p>Choose to delete or quarantine suspicious files with a single click</p>
                    </div>
                  </div>
                </div>
              </div>

              <div class="actions">
                <button class="secondary" on:click={() => activeWelcomeStep = 0}>Back</button>
                <button class="primary" on:click={() => activeWelcomeStep = 2}>Next</button>
              </div>
            {:else}
              <h2>Set Up Your VirusTotal API Key</h2>
              <p>To use this app, you'll need a free VirusTotal API key.</p>

              <div class="welcome-content">
                <div class="api-key-setup">
                  <div class="api-instructions">
                    <h3>How to get your API key:</h3>
                    <ol>
                      <li>Visit <a href="https://www.virustotal.com/gui/join-us" target="_blank" rel="noopener">virustotal.com</a> and create a free account</li>
                      <li>Once logged in, go to your profile settings</li>
                      <li>Find your API key in the "API Key" section</li>
                      <li>Copy and paste it below</li>
                    </ol>
                  </div>

                  <div class="api-key-input">
                    <label for="api-key">Your VirusTotal API Key:</label>
                    <input
                      id="api-key"
                      type="password"
                      bind:value={settings.api_key}
                      placeholder="Enter your API key"
                    />
                    <p class="input-help">Your API key is stored locally and is only used to communicate with VirusTotal.</p>
                  </div>
                </div>
              </div>

              <div class="actions">
                <button class="secondary" on:click={() => activeWelcomeStep = 1}>Back</button>
                <button class="primary" on:click={completeWelcome} disabled={!settings.api_key}>Complete Setup</button>
              </div>
            {/if}
          </div>
        </div>
      {:else}
        <!-- Main App Content -->
        {#if activeTab === 'scanner'}
          <div class="card">
            <h2>File Scanner</h2>
            <p>Scan files for viruses using VirusTotal.</p>

            <div class="scanner-actions">
              <button class="primary" on:click={openFileDialog} disabled={!settings.api_key}>
                Select File to Scan
              </button>
              <button class="secondary" on:click={scanDownloadsFolder} disabled={!settings.api_key}>
                Scan Downloads Folder
              </button>
              <button class="secondary" on:click={isMonitoring ? stopMonitoring : startMonitoring} disabled={!settings.api_key}>
                {isMonitoring ? 'Stop Monitoring' : 'Start Monitoring'}
              </button>
            </div>

            {#if activeScans.length > 0}
              <div class="active-scans">
                <h3>Active Scans</h3>
                <ul>
                  {#each activeScans as scan}
                    <li>
                      <div class="scan-item">
                        <div class="scan-info">
                          <span class="file-name">{scan.file_name}</span>
                          <span class="status">{scan.status}</span>
                        </div>
                        <div class="progress-bar">
                          <div class="progress" style="width: {scan.status === 'Pending' ? '10%' : scan.status === 'InProgress' ? '50%' : '100%'}"></div>
                        </div>
                      </div>
                    </li>
                  {/each}
                </ul>
              </div>
            {/if}
          </div>
        {:else if activeTab === 'history'}
          <div class="card">
            <h2>Scan History</h2>
            <p>View your previous scan results.</p>

            {#if scanHistory.length === 0}
              <div class="empty-state">
                <p>No scan history available.</p>
              </div>
            {:else}
              <div class="history-list">
                <table>
                  <thead>
                    <tr>
                      <th>File Name</th>
                      <th>Date</th>
                      <th>Status</th>
                      <th>Detections</th>
                    </tr>
                  </thead>
                  <tbody>
                    {#each scanHistory as scan}
                      <tr class={scan.status === 'Malicious' ? 'danger' : scan.status === 'Suspicious' ? 'warning' : 'success'}>
                        <td>{scan.file_name}</td>
                        <td>{new Date(scan.scan_date).toLocaleString()}</td>
                        <td>{scan.status}</td>
                        <td>
                          {#if scan.detection_count !== null && scan.total_engines !== null}
                            {scan.detection_count}/{scan.total_engines}
                          {:else}
                            -
                          {/if}
                        </td>
                      </tr>
                    {/each}
                  </tbody>
                </table>
              </div>
            {/if}
          </div>
        {:else if activeTab === 'quarantine'}
          <div class="card">
            <h2>Quarantined Files</h2>
            <p>Manage files that have been quarantined.</p>

            {#if quarantinedFiles.length === 0}
              <div class="empty-state">
                <p>No quarantined files available.</p>
              </div>
            {:else}
              <div class="quarantine-list">
                <table>
                  <thead>
                    <tr>
                      <th>File Name</th>
                      <th>Date</th>
                      <th>Detections</th>
                      <th>Actions</th>
                    </tr>
                  </thead>
                  <tbody>
                    {#each quarantinedFiles as file}
                      <tr>
                        <td>{file.file_name}</td>
                        <td>{new Date(file.quarantine_date).toLocaleString()}</td>
                        <td>{file.detection_count}/{file.total_engines}</td>
                        <td>
                          <button class="small" on:click={() => restoreFile(file.id)}>Restore</button>
                          <button class="small danger" on:click={() => deleteFile(file.id)}>Delete</button>
                        </td>
                      </tr>
                    {/each}
                  </tbody>
                </table>
              </div>
            {/if}
          </div>
        {:else if activeTab === 'settings'}
          <div class="card">
            <h2>Settings</h2>
            <p>Configure the application settings.</p>

            <div class="settings-form">
              <div class="form-group">
                <label for="api_key">VirusTotal API Key</label>
                <input type="password" id="api_key" bind:value={settings.api_key} placeholder="Enter your API key" />
              </div>

              <div class="form-group">
                <label for="theme">Theme</label>
                <select id="theme" bind:value={settings.theme}>
                  <option value="system">System Default</option>
                  <option value="light">Light</option>
                  <option value="dark">Dark</option>
                </select>
              </div>

              <div class="form-group checkbox">
                <input type="checkbox" id="auto_scan_downloads" bind:checked={settings.auto_scan_downloads} />
                <label for="auto_scan_downloads">Automatically scan new downloads</label>
              </div>

              <div class="form-group checkbox">
                <input type="checkbox" id="notify_on_scan_completion" bind:checked={settings.notify_on_scan_completion} />
                <label for="notify_on_scan_completion">Show notifications when scans complete</label>
              </div>

              <div class="form-group checkbox">
                <input type="checkbox" id="auto_quarantine_malicious" bind:checked={settings.auto_quarantine_malicious} />
                <label for="auto_quarantine_malicious">Automatically quarantine malicious files</label>
              </div>

              <div class="form-group checkbox">
                <input type="checkbox" id="minimize_to_tray" bind:checked={settings.minimize_to_tray} />
                <label for="minimize_to_tray">Minimize to system tray when closed</label>
              </div>

              <div class="form-group checkbox">
                <input type="checkbox" id="startup_with_system" bind:checked={settings.startup_with_system} />
                <label for="startup_with_system">Start with system</label>
              </div>

              <div class="form-group">
                <label for="scan_history_limit">Scan History Limit</label>
                <input type="number" id="scan_history_limit" bind:value={settings.scan_history_limit} min="10" max="1000" />
              </div>

              <div class="actions">
                <button class="primary" on:click={saveSettings}>Save Settings</button>
              </div>
            </div>
          </div>
        {/if}
      {/if}
    </div>

    {#if showNotification}
      <div class="notification {notificationType}">
        <p>{notificationMessage}</p>
        <button class="close" on:click={() => showNotification = false}>×</button>
      </div>
    {/if}

    {#if loading}
      <div class="loading-overlay">
        <div class="spinner"></div>
        <p>Processing...</p>
      </div>
    {/if}

    <footer>
      <p>VirusTotal Scanner App - Version 1.0.0</p>
    </footer>
  </div>
</main>

<style>
  :global(:root) {
    --bg-color: #f8fafc;
    --nav-bg: white;
    --card-bg: white;
    --text-color: #1e293b;
    --text-muted: #64748b;
    --border-color: #e2e8f0;
    --primary-color: #4361ee;
    --primary-hover: #3a56e4;
    --secondary-color: #64748b;
    --secondary-hover: #475569;
    --success-color: #2ecc71;
    --warning-color: #f39c12;
    --danger-color: #e74c3c;
    --shadow: 0 4px 12px rgba(0, 0, 0, 0.08);
    --border-radius: 12px;
    --transition-speed: 0.3s;
    --font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, 'Open Sans', 'Helvetica Neue', sans-serif;
    --content-width: 1200px;
    --header-height: 70px;
    --card-padding: 1.75rem;
    --input-height: 48px;
    --button-height: 48px;
  }

  :global(.dark-mode) {
    --bg-color: #0f172a;
    --nav-bg: #1e293b;
    --card-bg: #1e293b;
    --text-color: #f1f5f9;
    --text-muted: #94a3b8;
    --border-color: #334155;
    --primary-color: #4361ee;
    --primary-hover: #60a5fa;
    --secondary-color: #64748b;
    --secondary-hover: #94a3b8;
    --success-color: #2ecc71;
    --warning-color: #f39c12;
    --danger-color: #e74c3c;
    --shadow: 0 4px 16px rgba(0, 0, 0, 0.5);
    --card-shadow: 0 8px 24px rgba(0, 0, 0, 0.3);
  }

  :global(body) {
    background-color: var(--bg-color);
    color: var(--text-color);
    font-family: var(--font-family);
    margin: 0;
    padding: 0;
    transition: all var(--transition-speed) ease;
    line-height: 1.6;
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
  }

  :global(*, *::before, *::after) {
    box-sizing: border-box;
  }

  :global(h1, h2, h3, h4, h5, h6) {
    margin-top: 0;
    line-height: 1.2;
    font-weight: 600;
    color: var(--text-color);
  }

  :global(a) {
    color: var(--primary-color);
    text-decoration: none;
    transition: color var(--transition-speed) ease;
  }

  :global(a:hover) {
    color: var(--primary-hover);
    text-decoration: underline;
  }

  main {
    min-height: 100vh;
    background: var(--bg-color);
    display: flex;
    flex-direction: column;
  }

  .container {
    max-width: var(--content-width);
    width: 100%;
    margin: 0 auto;
    padding: 0 1.5rem;
    flex: 1;
  }

  header {
    display: flex;
    flex-direction: column;
    align-items: center;
    margin-bottom: 2.5rem;
    padding: 1.5rem 0;
    border-bottom: 1px solid var(--border-color);
    background-color: var(--nav-bg);
    box-shadow: var(--shadow);
    position: sticky;
    top: 0;
    z-index: 100;
    transition: all var(--transition-speed) ease;
  }

  h1 {
    color: var(--primary-color);
    margin-bottom: 1.25rem;
    font-size: 2rem;
    font-weight: 700;
    letter-spacing: -0.5px;
  }

  nav {
    width: 100%;
    max-width: var(--content-width);
  }

  nav ul {
    display: flex;
    list-style: none;
    padding: 0;
    margin: 0;
    gap: 0.75rem;
    flex-wrap: wrap;
    justify-content: center;
  }

  nav li {
    margin: 0;
  }

  nav button {
    background: transparent;
    color: var(--text-color);
    border: none;
    padding: 0.75rem 1.25rem;
    border-radius: var(--border-radius);
    cursor: pointer;
    font-weight: 500;
    transition: all var(--transition-speed) ease;
    position: relative;
    overflow: hidden;
  }

  nav button::after {
    content: '';
    position: absolute;
    bottom: 0;
    left: 50%;
    transform: translateX(-50%);
    width: 0;
    height: 3px;
    background-color: var(--primary-color);
    transition: width var(--transition-speed) ease;
  }

  nav li.active button {
    background: rgba(67, 97, 238, 0.1);
    color: var(--primary-color);
    font-weight: 600;
  }

  nav li.active button::after {
    width: 80%;
  }

  nav button:hover {
    background: rgba(67, 97, 238, 0.05);
  }

  nav button:hover::after {
    width: 40%;
  }

  nav li.active button:hover {
    background: rgba(67, 97, 238, 0.15);
  }

  .icon-button {
    background: transparent;
    border: none;
    font-size: 1.25rem;
    cursor: pointer;
    padding: 0.25rem 0.5rem;
    border-radius: 4px;
  }

  .content {
    margin-bottom: 2.5rem;
    padding-top: 1rem;
  }

  .card {
    background: var(--card-bg);
    border-radius: var(--border-radius);
    padding: var(--card-padding);
    box-shadow: var(--shadow);
    margin-bottom: 2rem;
    transition: all var(--transition-speed) ease;
    border: 1px solid var(--border-color);
    overflow: hidden;
  }

  .card:hover {
    transform: translateY(-3px);
    box-shadow: var(--card-shadow, 0 8px 24px rgba(0, 0, 0, 0.12));
  }

  .card h2 {
    font-size: 1.5rem;
    margin-bottom: 1rem;
    color: var(--primary-color);
    position: relative;
    display: inline-block;
  }

  .card h2::after {
    content: '';
    position: absolute;
    bottom: -5px;
    left: 0;
    width: 40px;
    height: 3px;
    background-color: var(--primary-color);
    border-radius: 3px;
  }

  .welcome-card {
    max-width: 800px;
    margin: 0 auto;
  }

  .welcome-steps {
    padding: 1rem;
  }

  .step-indicators {
    display: flex;
    justify-content: center;
    gap: 0.75rem;
    margin-bottom: 2rem;
  }

  .step-indicator {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background-color: var(--border-color);
    transition: all 0.3s ease;
  }

  .step-indicator.active {
    background-color: var(--primary-color);
    transform: scale(1.2);
  }

  .welcome-content {
    margin: 2rem 0;
  }

  .welcome-image {
    display: flex;
    justify-content: center;
    margin-bottom: 1.5rem;
  }

  .emoji-icon {
    font-size: 4rem;
  }

  .welcome-text ul {
    padding-left: 1.5rem;
  }

  .welcome-text li {
    margin-bottom: 0.5rem;
  }

  .how-it-works {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  .how-it-works .step {
    display: flex;
    align-items: flex-start;
    gap: 1rem;
  }

  .step-number {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 2rem;
    height: 2rem;
    background-color: var(--primary-color);
    color: white;
    border-radius: 50%;
    font-weight: bold;
  }

  .step-content {
    flex: 1;
  }

  .step-content h4 {
    margin-top: 0;
    margin-bottom: 0.5rem;
  }

  .step-content p {
    margin: 0;
  }

  .api-key-setup {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  .api-instructions ol {
    padding-left: 1.5rem;
  }

  .api-instructions li {
    margin-bottom: 0.5rem;
  }

  .api-key-input {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .input-help {
    font-size: 0.875rem;
    color: var(--text-muted);
  }

  h2 {
    margin-top: 0;
    margin-bottom: 1rem;
    color: var(--text-color);
  }

  h3 {
    margin-top: 1.5rem;
    margin-bottom: 0.75rem;
    color: var(--text-color);
  }

  p {
    margin-top: 0;
    margin-bottom: 1rem;
    color: var(--text-muted);
  }

  button {
    background: var(--primary-color);
    color: white;
    border: none;
    border-radius: var(--border-radius);
    padding: 0.75rem 1.5rem;
    font-size: 1rem;
    cursor: pointer;
    transition: all var(--transition-speed) ease;
    font-weight: 500;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
    height: var(--button-height);
    display: inline-flex;
    align-items: center;
    justify-content: center;
    position: relative;
    overflow: hidden;
  }

  button::after {
    content: '';
    position: absolute;
    top: 50%;
    left: 50%;
    width: 5px;
    height: 5px;
    background: rgba(255, 255, 255, 0.5);
    opacity: 0;
    border-radius: 100%;
    transform: scale(1, 1) translate(-50%, -50%);
    transform-origin: 50% 50%;
  }

  button:focus:not(:active)::after {
    animation: ripple 1s ease-out;
  }

  @keyframes ripple {
    0% {
      transform: scale(0, 0);
      opacity: 0.5;
    }
    20% {
      transform: scale(25, 25);
      opacity: 0.3;
    }
    100% {
      opacity: 0;
      transform: scale(40, 40);
    }
  }

  button:hover {
    background: var(--primary-hover);
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  }

  button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .primary {
    background: var(--primary-color);
  }

  .primary:hover {
    background: var(--primary-hover);
  }

  .secondary {
    background: var(--secondary-color);
  }

  .secondary:hover {
    background: var(--secondary-hover);
  }

  .danger {
    background: var(--danger-color);
  }

  .danger:hover {
    background: var(--danger-color);
    opacity: 0.9;
  }

  .small {
    padding: 0.25rem 0.5rem;
    font-size: 0.875rem;
  }

  .features {
    margin: 1.5rem 0;
  }

  .input-group {
    display: flex;
    margin-bottom: 1.5rem;
    gap: 0.5rem;
  }

  input, select {
    flex: 1;
    padding: 0.75rem 1rem;
    border: 1px solid var(--border-color);
    border-radius: var(--border-radius);
    font-size: 1rem;
    background-color: var(--card-bg);
    color: var(--text-color);
    height: var(--input-height);
    transition: all var(--transition-speed) ease;
    box-shadow: inset 0 1px 2px rgba(0, 0, 0, 0.05);
  }

  input:focus, select:focus {
    outline: none;
    border-color: var(--primary-color);
    box-shadow: 0 0 0 3px rgba(67, 97, 238, 0.15);
  }

  select {
    appearance: none;
    background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='16' height='16' viewBox='0 0 24 24' fill='none' stroke='%2364748b' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'%3E%3Cpolyline points='6 9 12 15 18 9'%3E%3C/polyline%3E%3C/svg%3E");
    background-repeat: no-repeat;
    background-position: right 1rem center;
    padding-right: 2.5rem;
  }

  input[type="checkbox"] {
    flex: 0 0 auto;
    width: 1.25rem;
    height: 1.25rem;
    margin-right: 0.5rem;
  }

  .setup-form {
    margin: 1.5rem 0;
  }

  .form-group {
    margin-bottom: 1.75rem;
  }

  .form-group label {
    display: block;
    margin-bottom: 0.5rem;
    color: var(--text-color);
    font-weight: 500;
    font-size: 0.95rem;
  }

  .input-help {
    margin-top: 0.5rem;
    font-size: 0.85rem;
    color: var(--text-muted);
  }

  .form-group.checkbox {
    display: flex;
    align-items: center;
    padding: 0.5rem 0;
  }

  .form-group.checkbox label {
    margin-bottom: 0;
    cursor: pointer;
  }

  .actions {
    display: flex;
    gap: 1rem;
    margin-top: 1.5rem;
  }

  .scanner-actions {
    display: flex;
    flex-wrap: wrap;
    gap: 0.75rem;
    margin: 1.5rem 0;
  }

  .active-scans {
    margin-top: 1.5rem;
  }

  .active-scans ul {
    list-style: none;
    padding: 0;
    margin: 0;
  }

  .active-scans li {
    margin-bottom: 1rem;
  }

  .scan-item {
    background: var(--bg-color);
    border-radius: 4px;
    padding: 1rem;
    border: 1px solid var(--border-color);
  }

  .scan-info {
    display: flex;
    justify-content: space-between;
    margin-bottom: 0.5rem;
  }

  .file-name {
    font-weight: bold;
    color: var(--text-color);
  }

  .status {
    color: var(--text-muted);
  }

  .progress-bar {
    height: 0.75rem;
    background: var(--border-color);
    border-radius: var(--border-radius);
    overflow: hidden;
    box-shadow: inset 0 1px 3px rgba(0, 0, 0, 0.1);
    width: 100%;
  }

  .progress {
    height: 100%;
    background: linear-gradient(90deg, var(--primary-color), var(--primary-hover));
    transition: width 0.3s ease;
    background-size: 200% 100%;
    animation: gradient-shift 2s ease infinite;
  }

  @keyframes gradient-shift {
    0% { background-position: 0% 50%; }
    50% { background-position: 100% 50%; }
    100% { background-position: 0% 50%; }
  }

  .history-list, .quarantine-list {
    overflow-x: auto;
    border-radius: var(--border-radius);
    box-shadow: var(--shadow);
    background-color: var(--card-bg);
    border: 1px solid var(--border-color);
    margin-bottom: 2rem;
  }

  table {
    width: 100%;
    border-collapse: separate;
    border-spacing: 0;
    margin-bottom: 0;
  }

  th, td {
    padding: 1rem;
    text-align: left;
    border-bottom: 1px solid var(--border-color);
    transition: all var(--transition-speed) ease;
  }

  th:first-child, td:first-child {
    padding-left: 1.5rem;
  }

  th:last-child, td:last-child {
    padding-right: 1.5rem;
  }

  th {
    font-weight: 600;
    color: var(--text-color);
    background-color: rgba(0, 0, 0, 0.02);
    position: sticky;
    top: 0;
    z-index: 10;
  }

  tbody tr {
    transition: all var(--transition-speed) ease;
  }

  tbody tr:hover td {
    background-color: rgba(0, 0, 0, 0.03);
  }

  tr.success td {
    background-color: rgba(16, 185, 129, 0.05);
  }

  tr.success:hover td {
    background-color: rgba(16, 185, 129, 0.1);
  }

  tr.warning td {
    background-color: rgba(245, 158, 11, 0.05);
  }

  tr.warning:hover td {
    background-color: rgba(245, 158, 11, 0.1);
  }

  tr.danger td {
    background-color: rgba(239, 68, 68, 0.05);
  }

  tr.danger:hover td {
    background-color: rgba(239, 68, 68, 0.1);
  }

  .empty-state {
    text-align: center;
    padding: 3rem 2rem;
    color: var(--text-muted);
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 1rem;
  }

  .empty-state svg {
    width: 64px;
    height: 64px;
    opacity: 0.5;
    margin-bottom: 1rem;
  }

  .empty-state h3 {
    margin: 0;
    color: var(--text-color);
    font-weight: 600;
  }

  .empty-state p {
    margin: 0;
    max-width: 400px;
  }

  .notification {
    position: fixed;
    bottom: 2rem;
    right: 2rem;
    padding: 1.25rem 1.5rem;
    border-radius: var(--border-radius);
    background: var(--card-bg);
    box-shadow: var(--shadow);
    display: flex;
    align-items: center;
    justify-content: space-between;
    max-width: 400px;
    z-index: 1000;
    animation: slide-in 0.3s cubic-bezier(0.175, 0.885, 0.32, 1.275);
    border-left: 4px solid var(--primary-color);
    gap: 1rem;
  }

  @keyframes slide-in {
    from { transform: translateX(100%); opacity: 0; }
    to { transform: translateX(0); opacity: 1; }
  }

  .notification p {
    margin: 0;
    padding-right: 1rem;
  }

  .notification.info {
    border-left-color: var(--info-color);
  }

  .notification.success {
    border-left-color: var(--success-color);
  }

  .notification.warning {
    border-left-color: var(--warning-color);
  }

  .notification.error {
    border-left-color: var(--danger-color);
  }

  .notification .close {
    background: transparent;
    color: var(--text-muted);
    border: none;
    font-size: 1.25rem;
    padding: 0;
    cursor: pointer;
  }

  .loading-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.7);
    backdrop-filter: blur(4px);
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    color: white;
    animation: fade-in 0.3s ease-out;
  }

  @keyframes fade-in {
    from { opacity: 0; }
    to { opacity: 1; }
  }

  .spinner {
    width: 50px;
    height: 50px;
    border: 4px solid rgba(255, 255, 255, 0.3);
    border-radius: 50%;
    border-top-color: var(--primary-color);
    border-left-color: var(--primary-color);
    animation: spin 0.8s cubic-bezier(0.5, 0, 0.5, 1) infinite;
    margin-bottom: 1rem;
    box-shadow: 0 0 10px rgba(0, 0, 0, 0.2);
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  footer {
    margin-top: 3rem;
    padding-top: 1.5rem;
    text-align: center;
    color: var(--text-muted);
    font-size: 0.875rem;
    border-top: 1px solid var(--border-color);
  }

  @media (max-width: 768px) {
    .container {
      padding: 0.5rem;
    }

    .card {
      padding: 1rem;
    }

    nav ul {
      flex-wrap: wrap;
      justify-content: center;
    }

    .actions {
      flex-direction: column;
    }

    .scanner-actions {
      flex-direction: column;
    }
  }
</style>
