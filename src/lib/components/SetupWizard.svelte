<!-- SetupWizard.svelte -->
<script lang="ts">
  import { invoke, open } from '../tauri';

  let currentStep = 1;
  let settings = {
    api_key: '',
    auto_scan_downloads: true,
    notify_on_scan_completion: true,
    auto_quarantine_malicious: false,
    startup_with_system: false,
    minimize_to_tray: true,
    custom_scan_locations: [] as string[],
  };
  let isLoading = false;
  let apiKeyError = '';

  async function validateApiKey() {
    if (!settings.api_key) {
      apiKeyError = 'API key is required';
      return false;
    }

    try {
      isLoading = true;
      // Test API key with a dummy scan
      await invoke('test_api_key', { apiKey: settings.api_key });
      apiKeyError = '';
      return true;
    } catch (error) {
      apiKeyError = `Invalid API key: ${error}`;
      return false;
    } finally {
      isLoading = false;
    }
  }

  async function addScanLocation() {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: 'Select folder to monitor'
      });

      if (selected && !settings.custom_scan_locations.includes(selected as string)) {
        settings.custom_scan_locations = [...settings.custom_scan_locations, selected as string];
      }
    } catch (error) {
      console.error('Failed to add location:', error);
    }
  }

  async function removeScanLocation(index: number) {
    settings.custom_scan_locations = settings.custom_scan_locations.filter((_, i) => i !== index);
  }

  async function nextStep() {
    if (currentStep === 1) {
      if (await validateApiKey()) {
        currentStep++;
      }
    } else if (currentStep < 4) {
      currentStep++;
    } else {
      await finishSetup();
    }
  }

  function prevStep() {
    if (currentStep > 1) {
      currentStep--;
    }
  }

  async function finishSetup() {
    try {
      isLoading = true;
      await invoke('update_settings', { settings });
      await invoke('set_welcome_completed', { completed: true });
      window.location.reload();
    } catch (error) {
      console.error('Failed to save settings:', error);
    } finally {
      isLoading = false;
    }
  }
</script>

<div class="setup-wizard">
  <div class="wizard-header">
    <h1>Welcome to VirusTotal Scanner</h1>
    <div class="step-indicator">
      <div class="step {currentStep >= 1 ? 'active' : ''}">1</div>
      <div class="step-line"></div>
      <div class="step {currentStep >= 2 ? 'active' : ''}">2</div>
      <div class="step-line"></div>
      <div class="step {currentStep >= 3 ? 'active' : ''}">3</div>
      <div class="step-line"></div>
      <div class="step {currentStep >= 4 ? 'active' : ''}">4</div>
    </div>
  </div>

  <div class="wizard-content">
    {#if currentStep === 1}
      <div class="step-content">
        <h2>API Key Setup</h2>
        <p>To use VirusTotal Scanner, you need a VirusTotal API key.</p>
        <div class="form-group">
          <label for="api-key">VirusTotal API Key</label>
          <input
            type="password"
            id="api-key"
            bind:value={settings.api_key}
            class={apiKeyError ? 'error' : ''}
            placeholder="Enter your VirusTotal API key"
          />
          {#if apiKeyError}
            <span class="error-text">{apiKeyError}</span>
          {/if}
          <p class="help-text">
            Don't have an API key?
            <a href="https://www.virustotal.com/gui/join-us" target="_blank" rel="noopener">
              Sign up for VirusTotal
            </a>
          </p>
        </div>
      </div>
    {:else if currentStep === 2}
      <div class="step-content">
        <h2>Monitoring Setup</h2>
        <div class="form-group">
          <label class="checkbox-label">
            <input type="checkbox" bind:checked={settings.auto_scan_downloads} />
            Automatically monitor Downloads folder
          </label>
        </div>
        <div class="form-group">
          <h3>Additional Folders to Monitor</h3>
          <button class="secondary-button" on:click={addScanLocation}>
            Add Folder
          </button>
          {#if settings.custom_scan_locations.length > 0}
            <div class="locations-list">
              {#each settings.custom_scan_locations as location, i}
                <div class="location-item">
                  <span class="location-path">{location}</span>
                  <button class="icon-button" on:click={() => removeScanLocation(i)}>×</button>
                </div>
              {/each}
            </div>
          {/if}
        </div>
      </div>
    {:else if currentStep === 3}
      <div class="step-content">
        <h2>Protection Settings</h2>
        <div class="form-group">
          <label class="checkbox-label">
            <input type="checkbox" bind:checked={settings.notify_on_scan_completion} />
            Show notifications when scans complete
          </label>
        </div>
        <div class="form-group">
          <label class="checkbox-label">
            <input type="checkbox" bind:checked={settings.auto_quarantine_malicious} />
            Automatically quarantine malicious files
          </label>
        </div>
      </div>
    {:else if currentStep === 4}
      <div class="step-content">
        <h2>System Integration</h2>
        <div class="form-group">
          <label class="checkbox-label">
            <input type="checkbox" bind:checked={settings.startup_with_system} />
            Start with system
          </label>
        </div>
        <div class="form-group">
          <label class="checkbox-label">
            <input type="checkbox" bind:checked={settings.minimize_to_tray} />
            Minimize to system tray when closed
          </label>
        </div>
      </div>
    {/if}
  </div>

  <div class="wizard-footer">
    {#if currentStep > 1}
      <button class="secondary-button" on:click={prevStep} disabled={isLoading}>
        Back
      </button>
    {/if}
    <button class="primary-button" on:click={nextStep} disabled={isLoading}>
      {currentStep === 4 ? 'Finish' : 'Next'}
    </button>
  </div>
</div>

<style>
  .setup-wizard {
    max-width: 800px;
    margin: 0 auto;
    padding: 2rem;
    display: flex;
    flex-direction: column;
    gap: 2rem;
  }

  .wizard-header {
    text-align: center;
  }

  .step-indicator {
    display: flex;
    align-items: center;
    justify-content: center;
    margin-top: 2rem;
  }

  .step {
    width: 32px;
    height: 32px;
    border-radius: 50%;
    background: var(--muted-color, #ccc);
    color: white;
    display: flex;
    align-items: center;
    justify-content: center;
    font-weight: bold;
  }

  .step.active {
    background: var(--primary-color, #0066cc);
  }

  .step-line {
    flex: 1;
    height: 2px;
    background: var(--muted-color, #ccc);
    margin: 0 8px;
    max-width: 100px;
  }

  .wizard-content {
    background: var(--card-bg, white);
    border-radius: 12px;
    padding: 2rem;
    box-shadow: var(--shadow, 0 2px 4px rgba(0, 0, 0, 0.1));
  }

  .form-group {
    margin-bottom: 1.5rem;
  }

  .checkbox-label {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    cursor: pointer;
  }

  input[type="password"] {
    width: 100%;
    padding: 0.5rem;
    border: 1px solid var(--border-color, #ccc);
    border-radius: 4px;
    margin-top: 0.25rem;
  }

  input[type="password"].error {
    border-color: var(--error-color, #dc3545);
  }

  .error-text {
    color: var(--error-color, #dc3545);
    font-size: 0.875rem;
    margin-top: 0.25rem;
    display: block;
  }

  .help-text {
    font-size: 0.875rem;
    color: var(--text-muted, #666);
    margin-top: 0.5rem;
  }

  .locations-list {
    margin-top: 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .location-item {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    background: var(--bg-subtle, #f5f5f5);
    padding: 0.5rem;
    border-radius: 4px;
  }

  .location-path {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .wizard-footer {
    display: flex;
    justify-content: flex-end;
    gap: 1rem;
    margin-top: 2rem;
  }

  button {
    padding: 0.5rem 1.5rem;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-weight: 500;
    transition: background-color 0.2s;
  }

  button:disabled {
    opacity: 0.7;
    cursor: not-allowed;
  }

  .primary-button {
    background: var(--primary-color, #0066cc);
    color: white;
  }

  .primary-button:hover:not(:disabled) {
    background: var(--primary-color-dark, #0052a3);
  }

  .secondary-button {
    background: var(--secondary-color, #e9ecef);
    color: var(--text-color, #212529);
  }

  .secondary-button:hover:not(:disabled) {
    background: var(--secondary-color-dark, #dde2e6);
  }

  .icon-button {
    padding: 0.25rem 0.5rem;
    background: transparent;
    color: var(--text-muted, #666);
    border: none;
    cursor: pointer;
  }

  .icon-button:hover {
    color: var(--text-color, #212529);
  }
</style>
