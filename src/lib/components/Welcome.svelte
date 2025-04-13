<!-- Welcome.svelte -->
<script lang="ts">
  import { invoke } from '../tauri';
  import { createEventDispatcher } from 'svelte';

  const dispatch = createEventDispatcher();
  let currentStep = 0;
  let apiKey = '';

  const steps = [
    {
      title: 'Welcome to VirusTotal Scanner',
      content: `This application helps keep your system secure by automatically scanning new downloads with VirusTotal's extensive virus detection engines. The app runs in the background and monitors your Downloads folder for new files.`
    },
    {
      title: 'How it works',
      content: `1. When a new file appears in your Downloads folder, you'll get a notification
2. Choose whether to scan the file with VirusTotal
3. View detailed scan results from multiple antivirus engines
4. Optionally delete suspicious files directly from the results window`
    },
    {
      title: 'Set up your VirusTotal API Key',
      content: `To use this app, you'll need a VirusTotal API key. Here's how to get one:

1. Visit virustotal.com and create a free account
2. Once logged in, go to your profile settings
3. Find your API key in the "API Key" section
4. Paste it below to start using the app`
    }
  ];

  async function saveApiKey() {
    if (!apiKey) {
      alert('Please enter an API key');
      return;
    }

    try {
      await invoke('set_api_key', { apiKey });
      dispatch('complete');
    } catch (error) {
      alert('Failed to save API key: ' + error);
    }
  }

  function nextStep() {
    if (currentStep < steps.length - 1) {
      currentStep++;
    }
  }

  function prevStep() {
    if (currentStep > 0) {
      currentStep--;
    }
  }
</script>

<div class="welcome-container">
  <div class="welcome-card">
    <div class="step-indicator">
      {#each steps as _, i}
        <div class="step {i === currentStep ? 'active' : ''}"></div>
      {/each}
    </div>

    <h1>{steps[currentStep].title}</h1>

    <div class="content">
      {#if currentStep === steps.length - 1}
        <p>{steps[currentStep].content}</p>
        <div class="api-key-input">
          <input
            type="password"
            placeholder="Enter your VirusTotal API key"
            bind:value={apiKey}
          />
          <a
            href="https://www.virustotal.com/gui/join-us"
            target="_blank"
            rel="noopener"
          >
            Get an API key
          </a>
        </div>
      {:else}
        <p>{steps[currentStep].content}</p>
      {/if}
    </div>

    <div class="buttons">
      {#if currentStep > 0}
        <button on:click={prevStep} class="secondary">
          Back
        </button>
      {/if}

      {#if currentStep === steps.length - 1}
        <button on:click={saveApiKey} class="primary">
          Start Using App
        </button>
      {:else}
        <button on:click={nextStep} class="primary">
          Next
        </button>
      {/if}
    </div>
  </div>
</div>

<style>
  .welcome-container {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 1rem;
  }

  .welcome-card {
    background: white;
    border-radius: 12px;
    padding: 2rem;
    max-width: 600px;
    width: 100%;
    box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
  }

  .step-indicator {
    display: flex;
    justify-content: center;
    gap: 0.5rem;
    margin-bottom: 2rem;
  }

  .step {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: #e5e7eb;
    transition: all 0.2s;
  }

  .step.active {
    background: #2563eb;
    transform: scale(1.2);
  }

  h1 {
    text-align: center;
    color: #1e293b;
    margin-bottom: 1.5rem;
    font-size: 1.5rem;
  }

  .content {
    margin-bottom: 2rem;
    line-height: 1.6;
  }

  .api-key-input {
    margin-top: 1.5rem;
  }

  input {
    width: 100%;
    padding: 0.75rem;
    border: 1px solid #e5e7eb;
    border-radius: 6px;
    margin-bottom: 0.5rem;
  }

  a {
    color: #2563eb;
    text-decoration: none;
    font-size: 0.875rem;
  }

  a:hover {
    text-decoration: underline;
  }

  .buttons {
    display: flex;
    justify-content: flex-end;
    gap: 1rem;
  }

  button {
    padding: 0.75rem 1.5rem;
    border-radius: 6px;
    border: none;
    cursor: pointer;
    font-weight: 500;
    transition: all 0.2s;
  }

  .primary {
    background: #2563eb;
    color: white;
  }

  .primary:hover {
    background: #1d4ed8;
  }

  .secondary {
    background: #e5e7eb;
    color: #4b5563;
  }

  .secondary:hover {
    background: #d1d5db;
  }
</style>
