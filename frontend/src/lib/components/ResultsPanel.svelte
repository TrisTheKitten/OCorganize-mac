<script lang="ts">
  import { fade, fly } from "svelte/transition";
  import type { PreviewResult } from "../api/organizer";
  import { formatCount } from "../utils/format";
  import CategoryTable from "./CategoryTable.svelte";
  import Icon from "./icons.svelte";
  import SummaryBar from "./SummaryBar.svelte";

  export type ResultsState = "empty" | "loading" | "preview" | "complete" | "error";

  export interface CompleteSummary {
    title: string;
    organizedCount: number;
    categoryCount: number;
    symlinkSkipCount: number;
    undoFile?: string | null;
    partial?: boolean;
    failedCount?: number;
  }

  interface Props {
    state: ResultsState;
    status: string;
    progress: number;
    preview: PreviewResult | null;
    complete: CompleteSummary | null;
    errorMessage: string;
  }

  let { state, status, progress, preview, complete, errorMessage }: Props = $props();

  const progressPercent = $derived(Math.round(progress * 100));
</script>

<div class="results-panel panel">
  <div class="panel-header">
    <h2 class="panel-title">Results</h2>
    {#if state === "loading"}
      <span class="status-badge loading">{status}</span>
    {:else if state === "error"}
      <span class="status-badge error">Error</span>
    {:else if state === "complete"}
      <span class="status-badge success">Complete</span>
    {:else if state === "preview"}
      <span class="status-badge preview">Preview</span>
    {/if}
  </div>

  {#if state === "loading"}
    <div class="loading-state" in:fade={{ duration: 200 }}>
      <p class="status-text">{status}</p>
      <div class="progress-track" role="progressbar" aria-valuenow={progressPercent} aria-valuemin={0} aria-valuemax={100}>
        <div class="progress-fill" style="width: {progressPercent}%"></div>
      </div>
    </div>
  {:else if state === "error"}
    <div class="message-state error-state" in:fly={{ y: 8, duration: 240 }}>
      <span class="state-icon error-icon"><Icon name="warning" size={22} /></span>
      <p>{errorMessage}</p>
    </div>
  {:else if state === "preview" && preview}
    <div class="preview-state" in:fade={{ duration: 200 }}>
      <SummaryBar {preview} />
      <CategoryTable categories={preview.categories} />
    </div>
  {:else if state === "complete" && complete}
    <div class="message-state complete-state" in:fly={{ y: 8, duration: 240 }}>
      <span class="state-icon complete-icon"><Icon name="check" size={22} /></span>
      <div class="complete-content">
        <p class="complete-title">{complete.title}</p>
        <ul class="complete-details">
          <li>{formatCount(complete.organizedCount)} files processed</li>
          {#if complete.categoryCount > 0}
            <li>{formatCount(complete.categoryCount)} folders</li>
          {/if}
          {#if complete.symlinkSkipCount > 0}
            <li>{formatCount(complete.symlinkSkipCount)} symlinks skipped</li>
          {/if}
          {#if complete.partial && complete.failedCount !== undefined}
            <li class="warn">{formatCount(complete.failedCount)} files failed</li>
          {/if}
          {#if complete.undoFile}
            <li class="mono">Undo: {complete.undoFile}</li>
          {/if}
        </ul>
      </div>
    </div>
  {:else}
    <div class="empty-state" in:fade={{ duration: 200 }}>
      <span class="empty-icon"><Icon name="folder" size={28} /></span>
      <p>Select a folder and run a preview.</p>
    </div>
  {/if}
</div>

<style>
  .results-panel {
    display: flex;
    flex-direction: column;
    min-height: 0;
    height: 100%;
    overflow: hidden;
  }

  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-2);
    margin-bottom: var(--space-3);
    flex-shrink: 0;
  }

  .panel-title {
    margin: 0;
    font-size: var(--text-md);
    font-weight: 600;
    color: var(--text-primary);
  }

  .status-badge {
    font-size: var(--text-xs);
    font-weight: 500;
    padding: 2px var(--space-1);
    border-radius: var(--radius-sm);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .status-badge.loading {
    color: var(--accent);
    background: var(--accent-soft);
  }

  .status-badge.error {
    color: var(--status-error);
    background: var(--status-error-bg);
  }

  .status-badge.success {
    color: var(--status-success);
    background: var(--status-success-bg);
  }

  .status-badge.preview {
    color: var(--status-neutral);
    background: var(--status-neutral-bg);
  }

  .loading-state {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .status-text {
    margin: 0;
    font-size: var(--text-sm);
    color: var(--text-secondary);
  }

  .progress-track {
    height: 6px;
    background: var(--surface-inset);
    border-radius: 999px;
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    background: var(--accent);
    border-radius: 999px;
    transition: width var(--duration-fast) var(--ease-out);
  }

  .empty-state,
  .message-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--space-2);
    padding: var(--space-6);
    text-align: center;
    color: var(--text-tertiary);
  }

  .empty-state p,
  .message-state p {
    margin: 0;
    font-size: var(--text-sm);
    max-width: 280px;
    line-height: 1.5;
  }

  .empty-icon,
  .state-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 64px;
    height: 64px;
    border-radius: var(--radius-lg);
    background: var(--surface-inset);
    color: var(--text-tertiary);
    box-shadow: inset 0 0 0 1px var(--border-subtle);
  }

  .empty-state .empty-icon {
    animation: empty-breathe 4s var(--ease-out) infinite;
  }

  @keyframes empty-breathe {
    0%,
    100% {
      transform: translateY(0);
    }
    50% {
      transform: translateY(-4px);
    }
  }

  .error-icon {
    background: var(--status-error-bg);
    color: var(--status-error);
  }

  .error-state {
    color: var(--status-error);
  }

  .complete-state {
    align-items: center;
    text-align: left;
    flex-direction: row;
    gap: var(--space-2);
    padding: var(--space-4);
    background: var(--status-success-bg);
    border-radius: var(--radius-md);
    border: 1px solid var(--status-success-border);
  }

  .complete-icon {
    background: var(--surface-elevated);
    color: var(--status-success);
    flex-shrink: 0;
    box-shadow: inset 0 0 0 1px var(--status-success-border);
  }

  .complete-content {
    flex: 1;
    min-width: 0;
  }

  .complete-title {
    font-size: var(--text-md);
    font-weight: 600;
    color: var(--text-primary);
    margin: 0 0 var(--space-1);
  }

  .complete-details {
    margin: 0;
    padding-left: var(--space-2);
    font-size: var(--text-sm);
    color: var(--text-secondary);
    line-height: 1.6;
  }

  .complete-details .warn {
    color: var(--status-warning);
  }

  .complete-details .mono {
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    word-break: break-all;
  }

  .preview-state {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    min-height: 0;
    flex: 1;
    overflow-y: auto;
  }

  @media (prefers-reduced-motion: reduce) {
    .progress-fill {
      transition: none;
    }

    .empty-state .empty-icon {
      animation: none;
    }
  }
</style>
