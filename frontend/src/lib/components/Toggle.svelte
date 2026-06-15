<script lang="ts">
  interface Props {
    checked: boolean;
    disabled?: boolean;
    label: string;
    onchange: (checked: boolean) => void;
  }

  let { checked = $bindable(false), disabled = false, label, onchange }: Props = $props();

  function handleChange(event: Event) {
    const next = (event.currentTarget as HTMLInputElement).checked;
    checked = next;
    onchange(next);
  }
</script>

<label class="toggle-row" class:disabled>
  <span class="toggle-label">{label}</span>
  <span class="toggle-switch">
    <input
      type="checkbox"
      role="switch"
      aria-label={label}
      {checked}
      {disabled}
      onchange={handleChange}
    />
    <span class="toggle-track" aria-hidden="true">
      <span class="toggle-thumb"></span>
    </span>
  </span>
</label>

<style>
  .toggle-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-2);
    min-height: 36px;
    padding: 0 4px;
    cursor: pointer;
    border-radius: var(--radius-sm);
    transition: background var(--duration-fast) var(--ease-out);
  }

  .toggle-row:hover:not(.disabled) {
    background: var(--surface-hover);
  }

  .toggle-row.disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .toggle-label {
    font-size: var(--text-sm);
    color: var(--text-primary);
    line-height: 1.4;
  }

  .toggle-switch {
    position: relative;
    flex-shrink: 0;
  }

  .toggle-switch input {
    position: absolute;
    inset: 0;
    opacity: 0;
    width: 100%;
    height: 100%;
    margin: 0;
    cursor: inherit;
    z-index: 1;
  }

  .toggle-track {
    display: block;
    width: 42px;
    height: 24px;
    border-radius: var(--radius-pill);
    background: var(--toggle-off);
    transition: background var(--duration-normal) var(--ease-out);
    position: relative;
  }

  .toggle-thumb {
    position: absolute;
    top: 2px;
    left: 2px;
    width: 20px;
    height: 20px;
    border-radius: 50%;
    background: var(--surface-elevated);
    box-shadow: var(--shadow-sm);
    transition: transform var(--duration-normal) var(--ease-spring);
  }

  .toggle-switch input:checked + .toggle-track {
    background: var(--accent);
  }

  .toggle-switch input:checked + .toggle-track .toggle-thumb {
    transform: translateX(18px);
  }

  .toggle-switch input:focus-visible + .toggle-track {
    box-shadow: 0 0 0 3px var(--accent-ring);
  }

  @media (prefers-reduced-motion: reduce) {
    .toggle-thumb,
    .toggle-track,
    .toggle-row {
      transition: none;
    }
  }
</style>
