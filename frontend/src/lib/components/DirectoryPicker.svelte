<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import Icon from "./icons.svelte";

  interface Props {
    value: string;
    disabled?: boolean;
    onchange: (path: string) => void;
    onerror: (message: string) => void;
  }

  let { value = $bindable(""), disabled = false, onchange, onerror }: Props = $props();

  let dragging = $state(false);

  async function browseDirectory() {
    const selected = await open({
      directory: true,
      multiple: false,
      title: "Select Directory to Organize",
      defaultPath: value || undefined,
    });
    if (typeof selected === "string") {
      onchange(selected);
    }
  }

  function handleDragOver(event: DragEvent) {
    event.preventDefault();
    if (!disabled) {
      dragging = true;
    }
  }

  function handleDragLeave() {
    dragging = false;
  }

  function handleDrop(event: DragEvent) {
    event.preventDefault();
    dragging = false;
    const file = event.dataTransfer?.files.item(0);
    if (!file) {
      return;
    }
    const path = (file as File & { path?: string }).path;
    if (!path) {
      onerror("Drop a folder from your file system.");
      return;
    }
    onchange(path);
  }
</script>

<section class="section">
  <h3 class="section-label">Directory</h3>
  <div
    class="drop-zone"
    class:dragging
    role="region"
    aria-label="Directory drop zone"
    ondragover={handleDragOver}
    ondragleave={handleDragLeave}
    ondrop={handleDrop}
  >
    <span class="drop-icon" aria-hidden="true">
      <Icon name="folder" size={18} />
    </span>
    <input
      bind:value
      placeholder="Select or drop a folder"
      {disabled}
      aria-label="Directory path"
      onchange={() => onchange(value)}
    />
    <button class="btn btn-secondary" type="button" {disabled} onclick={browseDirectory}>
      Browse
    </button>
  </div>
</section>

<style>
  .section {
    display: flex;
    flex-direction: column;
  }

  .drop-zone {
    display: flex;
    align-items: center;
    gap: var(--space-1);
    padding: 5px 5px 5px var(--space-1);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    background: var(--surface-inset);
    transition: border-color var(--duration-fast) var(--ease-out),
      background var(--duration-fast) var(--ease-out),
      box-shadow var(--duration-fast) var(--ease-out);
  }

  .drop-zone:focus-within {
    border-color: var(--accent);
    box-shadow: 0 0 0 3px var(--accent-ring);
  }

  .drop-zone.dragging {
    border-color: var(--accent);
    border-style: dashed;
    background: var(--accent-soft);
  }

  .drop-icon {
    display: flex;
    color: var(--text-tertiary);
    padding-left: 4px;
    flex-shrink: 0;
  }

  input {
    flex: 1;
    min-width: 0;
    border: none;
    background: transparent;
    padding: 4px 0;
    min-height: auto;
  }

  input:focus {
    border: none;
    outline: none;
    box-shadow: none;
  }
</style>
