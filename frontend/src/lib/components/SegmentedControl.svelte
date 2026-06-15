<script lang="ts" generics="T extends string">
  interface Option {
    id: T;
    label: string;
  }

  interface Props {
    options: Option[];
    value: T;
    disabled?: boolean;
    onchange: (value: T) => void;
  }

  let { options, value = $bindable(), disabled = false, onchange }: Props = $props();

  const activeIndex = $derived(Math.max(0, options.findIndex((option) => option.id === value)));
</script>

<div class="segmented" role="radiogroup" class:disabled style="--count: {options.length}; --active: {activeIndex}">
  <span class="indicator" aria-hidden="true"></span>
  {#each options as option (option.id)}
    <button
      type="button"
      class="segment"
      class:active={value === option.id}
      role="radio"
      aria-checked={value === option.id}
      {disabled}
      onclick={() => {
        value = option.id;
        onchange(option.id);
      }}
    >
      {option.label}
    </button>
  {/each}
</div>

<style>
  .segmented {
    position: relative;
    display: grid;
    grid-template-columns: repeat(var(--count), 1fr);
    padding: 3px;
    background: var(--surface-inset);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
  }

  .segmented.disabled {
    opacity: 0.5;
  }

  .indicator {
    position: absolute;
    top: 3px;
    left: 3px;
    height: calc(100% - 6px);
    width: calc((100% - 6px) / var(--count));
    background: var(--surface-elevated);
    border-radius: calc(var(--radius-md) - 3px);
    box-shadow: var(--shadow-sm);
    transform: translateX(calc(var(--active) * 100%));
    transition: transform var(--duration-normal) var(--ease-spring);
  }

  .segment {
    position: relative;
    z-index: 1;
    padding: 8px 10px;
    border: none;
    border-radius: calc(var(--radius-md) - 3px);
    background: transparent;
    color: var(--text-secondary);
    font-size: var(--text-sm);
    font-weight: var(--weight-medium);
    transition: color var(--duration-normal) var(--ease-out);
  }

  .segment:hover:not(:disabled):not(.active) {
    color: var(--text-primary);
  }

  .segment.active {
    color: var(--text-primary);
  }

  .segment:disabled {
    cursor: not-allowed;
  }

  .segment:focus-visible {
    box-shadow: 0 0 0 3px var(--accent-ring);
  }

  @media (prefers-reduced-motion: reduce) {
    .indicator {
      transition: none;
    }
  }
</style>
