<!-- ScanProgress.svelte -->
<script lang="ts">
  export let scanStep: number;
  export let scanStatusMessage: string;

  const steps = [
    { label: 'Preparing File', step: 1 },
    { label: 'Calculating Hash', step: 2 },
    { label: 'Uploading to VirusTotal', step: 3 },
    { label: 'Analyzing Results', step: 4 },
    { label: 'Generating Report', step: 5 }
  ];
</script>

<div class="scan-progress">
  <h3>Scanning in Progress</h3>

  <div class="progress-steps">
    {#each steps as { label, step }}
      <div class="step {scanStep >= step ? 'active' : ''}">
        <div class="step-icon">
          {#if scanStep > step}
            ✓
          {:else}
            {step}
          {/if}
        </div>
        <div class="step-label">{label}</div>
      </div>
    {/each}
  </div>

  <div class="progress-bar">
    <div class="progress" style="width: {(scanStep / steps.length) * 100}%"></div>
  </div>

  <p class="status-message">{scanStatusMessage}</p>
</div>

<style>
  .scan-progress {
    background: var(--card-bg);
    border-radius: 12px;
    padding: 2rem;
    margin: 2rem 0;
    box-shadow: var(--shadow);
  }

  h3 {
    margin: 0 0 2rem 0;
    text-align: center;
    color: var(--text-color);
  }

  .progress-steps {
    display: flex;
    justify-content: space-between;
    margin-bottom: 2rem;
    position: relative;
  }

  .progress-steps::before {
    content: '';
    position: absolute;
    top: 15px;
    left: 0;
    right: 0;
    height: 2px;
    background: var(--border-color);
    z-index: 0;
  }

  .step {
    display: flex;
    flex-direction: column;
    align-items: center;
    position: relative;
    z-index: 1;
    flex: 1;
    color: var(--text-muted);
    transition: color 0.3s;
  }

  .step.active {
    color: var(--text-color);
  }

  .step-icon {
    width: 32px;
    height: 32px;
    border-radius: 50%;
    background: var(--bg-subtle);
    border: 2px solid var(--border-color);
    display: flex;
    align-items: center;
    justify-content: center;
    margin-bottom: 0.5rem;
    font-weight: 600;
    transition: all 0.3s;
  }

  .step.active .step-icon {
    background: var(--primary-color);
    border-color: var(--primary-color);
    color: white;
  }

  .step-label {
    font-size: 0.875rem;
    text-align: center;
  }

  .progress-bar {
    height: 8px;
    background: var(--bg-subtle);
    border-radius: 4px;
    overflow: hidden;
    margin: 2rem 0;
  }

  .progress {
    height: 100%;
    background: var(--primary-color);
    transition: width 0.3s ease;
  }

  .status-message {
    text-align: center;
    color: var(--text-muted);
    margin: 0;
    font-style: italic;
  }
</style>
