<script lang="ts">
  interface Props {
    previewOnly: boolean;
    organizing: boolean;
    undoEnabled: boolean;
    onPreview: () => void;
    onOrganize: () => void;
    onUndo: () => void;
  }

  let {
    previewOnly,
    organizing,
    undoEnabled,
    onPreview,
    onOrganize,
    onUndo,
  }: Props = $props();
</script>

<section class="section">
  <h3 class="section-label">Actions</h3>
  <div class="primary-actions">
    <button class="btn btn-secondary" type="button" disabled={organizing} onclick={onPreview}>
      Preview
    </button>
    <button class="btn btn-primary" type="button" disabled={organizing} onclick={onOrganize}>
      {previewOnly ? "Run Preview" : "Organize Files"}
    </button>
  </div>
  <div class="undo-section">
    <button
      class="btn btn-danger"
      type="button"
      disabled={organizing || !undoEnabled}
      onclick={onUndo}
    >
      Undo Last
    </button>
  </div>
</section>

<style>
  .section {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .primary-actions {
    display: flex;
    gap: var(--space-1);
  }

  .primary-actions .btn {
    flex: 1;
    min-height: 40px;
  }

  .undo-section {
    padding-top: var(--space-2);
    border-top: 1px solid var(--border-subtle);
  }

  .undo-section .btn {
    width: 100%;
  }
</style>
