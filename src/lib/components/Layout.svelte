<!-- Layout.svelte -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '../tauri';
  import WelcomeWizard from './WelcomeWizard.svelte';
  import Scanner from './Scanner.svelte';
  import History from './History.svelte';
  import Quarantine from './Quarantine.svelte';
  import Settings from './Settings.svelte';
  import FileDetectionHandler from './FileDetectionHandler.svelte';
  import type { Settings as SettingsType } from '../types';

  let currentView: 'scanner' | 'history' | 'quarantine' | 'settings' = 'scanner';
  let settings: SettingsType;
  let showWelcome = true;
  let error: string | null = null;

  onMount(async () => {
    try {
      settings = await invoke('get_settings');
      showWelcome = !settings.welcome_completed;

      if (!showWelcome && settings.auto_scan_downloads) {
        await invoke('start_monitoring');
      }
    } catch (err) {
      error = `Failed to load application: ${err}`;
    }
  });

  function handleWelcomeComplete() {
    showWelcome = false;
  }
</script>

<div class="app-container" class:welcome-active={showWelcome}>
  {#if error}
    <div class="error-message">
      {error}
      <button class="retry-button" on:click={() => window.location.reload()}>
        Retry
      </button>
    </div>
  {/if}

  {#if showWelcome}
    <WelcomeWizard on:complete={handleWelcomeComplete} />
  {:else}
    <FileDetectionHandler
      onScanComplete={(result) => {
        // Handle scan complete if needed
      }}
      onTabChange={(tab) => {
        if (tab === 'scanner' || tab === 'history' || tab === 'quarantine' || tab === 'settings') {
          currentView = tab;
        }
      }}
    />
    <nav class="sidebar">
      <div class="nav-header">
        <img src="/tauri.svg" alt="Logo" class="logo" />
        <h1>VirusTotal Scanner</h1>
      </div>

      <div class="nav-items">
        <button
          class="nav-item {currentView === 'scanner' ? 'active' : ''}"
          on:click={() => currentView = 'scanner'}
        >
          <span class="icon">🔍</span>
          Scanner
        </button>

        <button
          class="nav-item {currentView === 'history' ? 'active' : ''}"
          on:click={() => currentView = 'history'}
        >
          <span class="icon">📊</span>
          History
        </button>

        <button
          class="nav-item {currentView === 'quarantine' ? 'active' : ''}"
          on:click={() => currentView = 'quarantine'}
        >
          <span class="icon">🔒</span>
          Quarantine
        </button>

        <button
          class="nav-item {currentView === 'settings' ? 'active' : ''}"
          on:click={() => currentView = 'settings'}
        >
          <span class="icon">⚙️</span>
          Settings
        </button>
      </div>
    </nav>

    <main class="content">
      {#if currentView === 'scanner'}
        <Scanner />
      {:else if currentView === 'history'}
        <History />
      {:else if currentView === 'quarantine'}
        <Quarantine />
      {:else if currentView === 'settings'}
        <Settings />
      {/if}
    </main>
  {/if}
</div>

<style>
  .app-container {
    display: flex;
    height: 100vh;
    width: 100vw;
    overflow: hidden;
    background: var(--bg-color);
    color: var(--text-color);
  }

  .app-container.welcome-active {
    display: block;
  }

  .error-message {
    position: fixed;
    top: 1rem;
    left: 50%;
    transform: translateX(-50%);
    background: var(--error-subtle);
    color: var(--error);
    padding: 1rem;
    border-radius: 8px;
    display: flex;
    align-items: center;
    gap: 1rem;
    z-index: 1000;
    box-shadow: var(--shadow);
  }

  .retry-button {
    padding: 0.5rem 1rem;
    background: var(--error);
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
  }

  .sidebar {
    width: 250px;
    background: var(--card-bg);
    border-right: 1px solid var(--border-color);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .nav-header {
    padding: 1.5rem;
    border-bottom: 1px solid var(--border-color);
    display: flex;
    align-items: center;
    gap: 1rem;
  }

  .logo {
    width: 32px;
    height: 32px;
  }

  h1 {
    margin: 0;
    font-size: 1.25rem;
    color: var(--text-color);
  }

  .nav-items {
    padding: 1rem 0;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .nav-item {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.75rem 1.5rem;
    background: none;
    border: none;
    color: var(--text-muted);
    font-size: 1rem;
    cursor: pointer;
    transition: all 0.2s;
    text-align: left;
    width: 100%;
  }

  .nav-item:hover {
    background: var(--bg-subtle);
    color: var(--text-color);
  }

  .nav-item.active {
    background: var(--primary-subtle);
    color: var(--primary-color);
    font-weight: 500;
  }

  .icon {
    font-size: 1.25rem;
  }

  .content {
    flex: 1;
    overflow-y: auto;
    padding: 2rem;
  }

  @media (max-width: 768px) {
    .sidebar {
      position: fixed;
      bottom: 0;
      left: 0;
      right: 0;
      width: 100%;
      height: auto;
      border-right: none;
      border-top: 1px solid var(--border-color);
      z-index: 100;
    }

    .nav-header {
      display: none;
    }

    .nav-items {
      padding: 0.5rem;
      flex-direction: row;
      justify-content: space-around;
    }

    .nav-item {
      flex-direction: column;
      padding: 0.5rem;
      gap: 0.25rem;
      font-size: 0.75rem;
    }

    .content {
      padding-bottom: 5rem;
    }
  }
</style>
