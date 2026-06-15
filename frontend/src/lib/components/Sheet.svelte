<script lang="ts">
  import { onMount } from "svelte";
  import type { Snippet } from "svelte";
  import { fade, scale } from "svelte/transition";
  import { cubicOut } from "svelte/easing";

  interface Props {
    open: boolean;
    title: string;
    onclose: () => void;
    body: Snippet;
    actions?: Snippet;
  }

  let { open, title, onclose, body, actions }: Props = $props();

  let sheetEl = $state<HTMLDivElement | null>(null);

  onMount(() => {
    function handleKeydown(event: KeyboardEvent) {
      if (event.key === "Escape" && open) {
        onclose();
      }
    }
    window.addEventListener("keydown", handleKeydown);
    return () => window.removeEventListener("keydown", handleKeydown);
  });

  $effect(() => {
    if (open && sheetEl) {
      sheetEl.focus();
    }
  });
</script>

{#if open}
  <div
    class="sheet-backdrop"
    role="presentation"
    onclick={onclose}
    transition:fade={{ duration: 200 }}
  ></div>
  <div class="sheet-wrap">
    <div
      class="sheet"
      role="dialog"
      aria-modal="true"
      aria-labelledby="sheet-title"
      tabindex="-1"
      bind:this={sheetEl}
      transition:scale={{ duration: 240, start: 0.96, opacity: 0, easing: cubicOut }}
    >
      <div class="sheet-header">
        <h2 id="sheet-title" class="sheet-title">{title}</h2>
        <button class="sheet-close" type="button" aria-label="Close" onclick={onclose}>
          <svg width="14" height="14" viewBox="0 0 16 16" fill="none" aria-hidden="true">
            <path d="M4 4L12 12M12 4L4 12" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
          </svg>
        </button>
      </div>
      <div class="sheet-body">
        {@render body()}
      </div>
      {#if actions}
        <div class="sheet-actions">
          {@render actions()}
        </div>
      {/if}
    </div>
  </div>
{/if}

<style>
  .sheet-backdrop {
    position: fixed;
    inset: 0;
    background: var(--backdrop);
    z-index: 100;
  }

  .sheet-wrap {
    position: fixed;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--space-3);
    z-index: 101;
    pointer-events: none;
  }

  .sheet {
    width: min(440px, 100%);
    max-height: calc(100vh - var(--space-6));
    display: flex;
    flex-direction: column;
    background: var(--surface-elevated);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-lg);
    outline: none;
    pointer-events: auto;
    overflow: hidden;
  }

  .sheet-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-2);
    padding: var(--space-3) var(--space-3) var(--space-2);
  }

  .sheet-title {
    margin: 0;
    font-size: var(--text-lg);
    font-weight: var(--weight-semibold);
    color: var(--text-primary);
  }

  .sheet-close {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    flex-shrink: 0;
    border: none;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--text-tertiary);
    transition: background var(--duration-fast) var(--ease-out),
      color var(--duration-fast) var(--ease-out);
  }

  .sheet-close:hover {
    background: var(--surface-hover);
    color: var(--text-primary);
  }

  .sheet-body {
    padding: 0 var(--space-3) var(--space-2);
    overflow-y: auto;
    font-size: var(--text-sm);
    color: var(--text-secondary);
    line-height: 1.55;
  }

  .sheet-actions {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-1);
    padding: var(--space-2) var(--space-3) var(--space-3);
  }

  @media (prefers-reduced-motion: reduce) {
    .sheet-backdrop,
    .sheet {
      transition: none;
    }
  }
</style>
