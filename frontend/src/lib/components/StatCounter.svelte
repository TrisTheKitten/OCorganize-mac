<script lang="ts">
  import { onMount } from "svelte";
  import { formatCount } from "../utils/format";

  interface Props {
    value: number;
    label: string;
  }

  let { value, label }: Props = $props();

  let displayValue = $state(0);
  let reducedMotion = $state(false);

  onMount(() => {
    reducedMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    displayValue = value;
  });

  $effect(() => {
    if (reducedMotion) {
      displayValue = value;
      return;
    }

    const start = displayValue;
    const end = value;
    if (start === end) {
      return;
    }

    const duration = 400;
    const startTime = performance.now();

    function tick(now: number) {
      const elapsed = now - startTime;
      const t = Math.min(elapsed / duration, 1);
      const eased = 1 - Math.pow(1 - t, 3);
      displayValue = Math.round(start + (end - start) * eased);
      if (t < 1) {
        requestAnimationFrame(tick);
      }
    }

    const frame = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(frame);
  });
</script>

<div class="stat">
  <span class="stat-value">{formatCount(displayValue)}</span>
  <span class="stat-label">{label}</span>
</div>

<style>
  .stat {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .stat-value {
    font-size: var(--text-2xl);
    font-weight: var(--weight-semibold);
    font-variant-numeric: tabular-nums;
    color: var(--text-primary);
    line-height: 1.1;
    letter-spacing: -0.02em;
  }

  .stat-label {
    font-size: var(--text-xs);
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }
</style>
