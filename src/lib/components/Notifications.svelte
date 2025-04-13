<!-- Notifications.svelte -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke, isPermissionGranted, requestPermission, sendNotification } from '../tauri';

  export let settings = {
    notify_on_scan_completion: true
  };

  let notificationsEnabled = false;

  onMount(async () => {
    if (settings.notify_on_scan_completion) {
      await setupNotifications();
    }
  });

  async function setupNotifications() {
    try {
      let permissionGranted = await isPermissionGranted();

      if (!permissionGranted) {
        const permission = await requestPermission();
        permissionGranted = permission === 'granted';
      }

      notificationsEnabled = permissionGranted;

      if (notificationsEnabled) {
        await invoke('enable_notifications');
      }
    } catch (error) {
      console.error('Failed to setup notifications:', error);
    }
  }
</script>

{#if !notificationsEnabled && settings.notify_on_scan_completion}
  <div class="notification-prompt">
    <div class="prompt-content">
      <span class="icon">🔔</span>
      <div class="prompt-text">
        <p>Enable notifications to get scan results and important alerts</p>
        <button on:click={setupNotifications}>Enable Notifications</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .notification-prompt {
    position: fixed;
    bottom: 2rem;
    right: 2rem;
    background: var(--card-bg);
    border-radius: 8px;
    padding: 1rem;
    box-shadow: var(--shadow-lg);
    z-index: 1000;
    animation: slideIn 0.3s ease-out;
    border: 1px solid var(--border-color);
  }

  @keyframes slideIn {
    from { transform: translateY(100%); opacity: 0; }
    to { transform: translateY(0); opacity: 1; }
  }

  .prompt-content {
    display: flex;
    align-items: center;
    gap: 1rem;
  }

  .icon {
    font-size: 1.5rem;
  }

  .prompt-text {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .prompt-text p {
    margin: 0;
    color: var(--text-color);
  }

  button {
    padding: 0.5rem 1rem;
    background: var(--primary-color);
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    transition: all 0.2s;
  }

  button:hover {
    transform: translateY(-1px);
    box-shadow: var(--shadow);
  }

  @media (max-width: 640px) {
    .notification-prompt {
      bottom: 5rem;
      left: 1rem;
      right: 1rem;
    }
  }
</style>
