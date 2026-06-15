<script lang="ts">
  import Toggle from "./Toggle.svelte";

  interface Props {
    createUndo: boolean;
    previewOnly: boolean;
    disabled?: boolean;
    onCreateUndoChange: (value: boolean) => void;
    onPreviewOnlyChange: (value: boolean) => void;
  }

  let {
    createUndo = $bindable(true),
    previewOnly = $bindable(false),
    disabled = false,
    onCreateUndoChange,
    onPreviewOnlyChange,
  }: Props = $props();
</script>

<section class="section">
  <h3 class="section-label">Options</h3>
  <div class="options">
    <Toggle
      bind:checked={createUndo}
      label="Create undo file"
      {disabled}
      onchange={onCreateUndoChange}
    />
    <Toggle
      bind:checked={previewOnly}
      label="Preview only"
      {disabled}
      onchange={onPreviewOnlyChange}
    />
  </div>
</section>

<style>
  .section {
    display: flex;
    flex-direction: column;
  }

  .options {
    display: flex;
    flex-direction: column;
    padding: 6px var(--space-1);
    background: var(--surface-inset);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
  }
</style>
