<!-- WelcomeWizard.svelte -->
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { invoke } from '../tauri.js';
  import type { Settings } from '../types';

  const dispatch = createEventDispatcher();
  let currentStep = 1;
  let apiKey = '';
  let isTestingKey = false;
  let keyError: string | null = null;
  let settings: Settings = {
    api_key: '',
    welcome_completed: false,
    auto_scan_downloads: true,
    notify_on_scan_completion: true,
    auto_quarantine_malicious: false,
    custom_scan_locations: [],
    scan_history_limit: 1000,
    file_type_filters: [
      'exe', 'dll', 'bat', 'cmd', 'ps1', 'vbs',
      'js', 'jar', 'msi', 'zip', 'rar', 'scr',
      'pdf', 'doc', 'docx'
    ],
    theme: 'system',
    startup_with_system: false,
    minimize_to_tray: true,
    background_scan_threads: 2
  };

  async function testApiKey() {
    try {
      isTestingKey = true;
      keyError = null;

      // Validate API key format first
      if (!apiKey || apiKey.length < 32) {
        keyError = 'API key is too short. VirusTotal API keys are typically 64 characters long.';
        isTestingKey = false;
        return;
      }

      // Test the API key with VirusTotal
      const isValid = await invoke('test_api_key', { apiKey });

      if (isValid) {
        settings.api_key = apiKey;
        currentStep++;
      } else {
        keyError = 'Invalid API key. Please check your key and try again.';
      }
    } catch (err) {
      keyError = `Error testing API key: ${err}`;
    } finally {
      isTestingKey = false;
    }
  }

  async function completeSetup() {
    try {
      settings.welcome_completed = true;
      await invoke('update_settings', { settings });

      if (settings.auto_scan_downloads) {
        await invoke('start_monitoring');
      }

      dispatch('complete');
    } catch (err) {
      console.error('Failed to save settings:', err);
    }
  }

  function handleNext() {
    if (currentStep === 1 && !apiKey) {
      keyError = 'Please enter your VirusTotal API key';
      return;
    }

    if (currentStep === 1) {
      testApiKey();
    } else if (currentStep === 3) {
      completeSetup();
    } else {
      currentStep++;
    }
  }

  function handleBack() {
    if (currentStep > 1) {
      currentStep--;
    }
  }
</script>

<div class="wizard-overlay">
  <div class="wizard-container">
    <div class="wizard-header">
      <div class="step-indicator">
        <div class="step {currentStep >= 1 ? 'active' : ''}">1</div>
        <div class="step-line"></div>
        <div class="step {currentStep >= 2 ? 'active' : ''}">2</div>
        <div class="step-line"></div>
        <div class="step {currentStep >= 3 ? 'active' : ''}">3</div>
      </div>
    </div>

    <div class="wizard-content">
      {#if currentStep === 1}
        <div class="step-content">
          <h2>Welcome to VirusTotal Scanner</h2>
          <p class="description">
            To get started, you'll need a VirusTotal API key. This key allows the application
            to scan files using VirusTotal's services.
          </p>

          <div class="input-group">
            <label for="api_key">VirusTotal API Key</label>
            <input
              type="text"
              id="api_key"
              bind:value={apiKey}
              placeholder="Enter your API key"
              class={keyError ? 'error' : ''}
            />
            {#if keyError}
              <span class="error-message">{keyError}</span>
            {/if}
          </div>

          <div class="help-text">
            <p>
              Don't have an API key? You can get one for free from VirusTotal:
            </p>
            <ol>
              <li>Visit <a href="https://www.virustotal.com/gui/join-us" target="_blank" rel="noopener noreferrer">VirusTotal</a></li>
              <li>Create a free account</li>
              <li>Go to your profile settings</li>
              <li>Copy your API key</li>
            </ol>
          </div>
        </div>

      {:else if currentStep === 2}
        <div class="step-content">
          <h2>Configure Scanning Options</h2>
          <p class="description">
            Choose how you want the application to handle file scanning and monitoring.
          </p>

          <div class="options-group">
            <label class="checkbox-label">
              <input
                type="checkbox"
                bind:checked={settings.auto_scan_downloads}
              />
              Automatically scan new downloads
            </label>
            <p class="option-description">
              Monitor your downloads folder and automatically scan new files
            </p>

            <label class="checkbox-label">
              <input
                type="checkbox"
                bind:checked={settings.notify_on_scan_completion}
              />
              Show notifications when scans complete
            </label>
            <p class="option-description">
              Receive desktop notifications with scan results
            </p>

            <label class="checkbox-label">
              <input
                type="checkbox"
                bind:checked={settings.auto_quarantine_malicious}
              />
              Automatically quarantine malicious files
            </label>
            <p class="option-description">
              Move detected malicious files to quarantine automatically
            </p>
          </div>
        </div>

      {:else if currentStep === 3}
        <div class="step-content">
          <h2>Application Settings</h2>
          <p class="description">
            Configure how you want the application to behave on your system.
          </p>

          <div class="options-group">
            <label class="checkbox-label">
              <input
                type="checkbox"
                bind:checked={settings.startup_with_system}
              />
              Start with system
            </label>
            <p class="option-description">
              Launch the application automatically when you start your computer
            </p>

            <label class="checkbox-label">
              <input
                type="checkbox"
                bind:checked={settings.minimize_to_tray}
              />
              Minimize to system tray
            </label>
            <p class="option-description">
              Keep the application running in the background when closed
            </p>

            <div class="select-group">
              <label for="theme">Theme</label>
              <select id="theme" bind:value={settings.theme}>
                <option value="system">System Default</option>
                <option value="light">Light</option>
                <option value="dark">Dark</option>
              </select>
            </div>
          </div>

          <div class="completion-message">
            <p>
              🎉 You're all set! Click "Complete" to start using VirusTotal Scanner.
            </p>
          </div>
        </div>
      {/if}
    </div>

    <div class="wizard-footer">
      {#if currentStep > 1}
        <button class="back-button" on:click={handleBack}>
          Back
        </button>
      {/if}

      <button
        class="next-button"
        on:click={handleNext}
        disabled={isTestingKey}
      >
        {#if isTestingKey}
          Testing...
        {:else if currentStep === 3}
          Complete
        {:else if currentStep === 1}
          Verify & Continue
        {:else}
          Next
        {/if}
      </button>
    </div>
  </div>
</div>

<style>
  .wizard-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .wizard-container {
    background: var(--card-bg);
    border-radius: 12px;
    box-shadow: var(--shadow-lg);
    width: 90%;
    max-width: 600px;
    max-height: 90vh;
    display: flex;
    flex-direction: column;
  }

  .wizard-header {
    padding: 1.5rem;
    border-bottom: 1px solid var(--border-color);
  }

  .step-indicator {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
  }

  .step {
    width: 32px;
    height: 32px;
    border-radius: 50%;
    background: var(--bg-subtle);
    border: 2px solid var(--border-color);
    display: flex;
    align-items: center;
    justify-content: center;
    font-weight: 500;
    color: var(--text-muted);
    transition: all 0.3s;
  }

  .step.active {
    background: var(--primary-color);
    border-color: var(--primary-color);
    color: white;
  }

  .step-line {
    flex: 1;
    height: 2px;
    background: var(--border-color);
    max-width: 100px;
  }

  .wizard-content {
    padding: 2rem;
    overflow-y: auto;
  }

  .step-content {
    animation: fadeIn 0.3s ease-out;
  }

  @keyframes fadeIn {
    from { opacity: 0; transform: translateY(10px); }
    to { opacity: 1; transform: translateY(0); }
  }

  h2 {
    margin: 0 0 1rem 0;
    color: var(--text-color);
  }

  .description {
    color: var(--text-muted);
    margin-bottom: 2rem;
  }

  .input-group {
    margin-bottom: 1.5rem;
  }

  label {
    display: block;
    margin-bottom: 0.5rem;
    color: var(--text-color);
  }

  input[type="text"] {
    width: 100%;
    padding: 0.75rem;
    border: 1px solid var(--border-color);
    border-radius: 6px;
    background: var(--bg-subtle);
    color: var(--text-color);
  }

  input[type="text"].error {
    border-color: var(--error);
  }

  .error-message {
    color: var(--error);
    font-size: 0.875rem;
    margin-top: 0.5rem;
    display: block;
  }

  .help-text {
    font-size: 0.875rem;
    color: var(--text-muted);
  }

  .help-text a {
    color: var(--primary-color);
    text-decoration: none;
  }

  .help-text a:hover {
    text-decoration: underline;
  }

  .options-group {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  .checkbox-label {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    color: var(--text-color);
    font-weight: 500;
    cursor: pointer;
  }

  .option-description {
    margin: 0.25rem 0 0 1.75rem;
    font-size: 0.875rem;
    color: var(--text-muted);
  }

  .select-group {
    margin-top: 1rem;
  }

  select {
    width: 100%;
    padding: 0.75rem;
    border: 1px solid var(--border-color);
    border-radius: 6px;
    background: var(--bg-subtle);
    color: var(--text-color);
  }

  .completion-message {
    margin-top: 2rem;
    padding: 1rem;
    background: var(--success-subtle);
    border-radius: 8px;
    text-align: center;
  }

  .completion-message p {
    margin: 0;
    color: var(--success);
    font-weight: 500;
  }

  .wizard-footer {
    padding: 1.5rem;
    border-top: 1px solid var(--border-color);
    display: flex;
    justify-content: flex-end;
    gap: 1rem;
  }

  .back-button,
  .next-button {
    padding: 0.75rem 1.5rem;
    border: none;
    border-radius: 6px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
  }

  .back-button {
    background: var(--bg-subtle);
    color: var(--text-color);
  }

  .next-button {
    background: var(--primary-color);
    color: white;
  }

  .back-button:hover,
  .next-button:hover {
    transform: translateY(-2px);
    box-shadow: var(--shadow);
  }

  .next-button:disabled {
    opacity: 0.7;
    cursor: not-allowed;
    transform: none;
  }

  @media (max-width: 640px) {
    .wizard-container {
      width: 95%;
      max-height: 95vh;
    }

    .wizard-content {
      padding: 1rem;
    }
  }
</style>
