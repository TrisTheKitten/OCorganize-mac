<script lang="ts">
  import type { PreviewResult } from "../api/organizer";
  import StatCounter from "./StatCounter.svelte";

  interface Props {
    preview: PreviewResult;
  }

  let { preview }: Props = $props();
</script>

<div class="summary-bar">
  <StatCounter value={preview.totalFiles} label="Files" />
  <span class="divider" aria-hidden="true"></span>
  <StatCounter value={preview.categoryCount} label="Categories" />
  {#if preview.symlinkSkipCount > 0}
    <span class="divider" aria-hidden="true"></span>
    <StatCounter value={preview.symlinkSkipCount} label="Symlinks skipped" />
  {/if}
</div>

<style>
  .summary-bar {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-3);
    background: var(--surface-inset);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
  }

  .divider {
    width: 1px;
    align-self: stretch;
    background: var(--border-subtle);
  }
</style>
