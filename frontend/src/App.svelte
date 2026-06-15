<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import DirectoryPicker from "./lib/components/DirectoryPicker.svelte";
  import StrategySelector from "./lib/components/StrategySelector.svelte";
  import OptionsPanel from "./lib/components/OptionsPanel.svelte";
  import ActionBar from "./lib/components/ActionBar.svelte";
  import ResultsPanel, {
    type CompleteSummary,
    type ResultsState,
  } from "./lib/components/ResultsPanel.svelte";
  import Sheet from "./lib/components/Sheet.svelte";
  import Icon from "./lib/components/icons.svelte";
  import {
    hasUndoOperations,
    onOrganizeProgress,
    organizeFiles,
    previewOrganization,
    undoLastOperation,
    validateDirectory,
    type PreviewResult,
    type Strategy,
  } from "./lib/api/organizer";
  import { previewSummaryLines } from "./lib/utils/previewSummary";

  type SheetState = "preview-complete" | "confirm-undo" | "confirm-organize" | null;

  let directory = $state("");
  let strategy = $state<Strategy>("type");
  let createUndo = $state(true);
  let previewOnly = $state(false);
  let organizing = $state(false);
  let undoEnabled = $state(false);
  let status = $state("Ready");
  let progress = $state(0);
  let previewResult = $state<PreviewResult | null>(null);
  let completeSummary = $state<CompleteSummary | null>(null);
  let errorMessage = $state("");
  let resultsState = $state<ResultsState>("empty");
  let sheetState = $state<SheetState>(null);

  onMount(() => {
    const unlisteners: Array<() => void> = [];

    onOrganizeProgress((event) => {
      status = event.status;
      progress = event.progress;
    }).then((unlisten) => unlisteners.push(unlisten));

    getCurrentWindow().onDragDropEvent((event) => {
      if (event.payload.type === "drop") {
        const paths = event.payload.paths;
        if (paths.length > 0) {
          setDirectory(paths[0]);
        }
      }
    }).then((unlisten) => unlisteners.push(unlisten));

    refreshUndoState();

    return () => {
      for (const unlisten of unlisteners) {
        unlisten();
      }
    };
  });

  async function refreshUndoState() {
    undoEnabled = await hasUndoOperations();
  }

  async function setDirectory(path: string) {
    try {
      directory = await validateDirectory(path);
      status = "Directory selected";
      if (resultsState === "error") {
        resultsState = "empty";
        errorMessage = "";
      }
    } catch (error) {
      showError(String(error));
    }
  }

  function showError(message: string) {
    status = "Error";
    errorMessage = message.replace(/^Error:\s*/i, "");
    resultsState = "error";
    progress = 0;
    previewResult = null;
    completeSummary = null;
  }

  async function runPreview() {
    if (!directory) {
      showError("Please select a directory.");
      return;
    }

    organizing = true;
    resultsState = "loading";
    status = "Analyzing files for preview...";
    progress = 0.1;
    previewResult = null;
    completeSummary = null;
    errorMessage = "";

    try {
      const preview = await previewOrganization(directory, strategy);

      if (preview.totalFiles === 0) {
        errorMessage =
          preview.symlinkSkipCount > 0
            ? `No regular files found (skipped ${preview.symlinkSkipCount} symlinked items).`
            : "No files found in the selected directory.";
        resultsState = "error";
        status = "No files available for preview";
        progress = 0;
        return;
      }

      previewResult = preview;
      resultsState = "preview";
      progress = 1;
      status = "Preview complete";
      sheetState = "preview-complete";
    } catch (error) {
      showError(String(error));
    } finally {
      organizing = false;
    }
  }

  function requestOrganize() {
    if (!directory) {
      showError("Please select a directory.");
      return;
    }
    if (previewOnly) {
      void runOrganize();
      return;
    }
    sheetState = "confirm-organize";
  }

  async function runOrganize() {
    sheetState = null;
    organizing = true;
    resultsState = "loading";
    status = previewOnly ? "Running preview..." : "Organizing files...";
    progress = 0.1;
    completeSummary = null;
    errorMessage = "";

    try {
      const summary = await organizeFiles(directory, strategy, previewOnly, createUndo);
      const action = summary.previewOnly ? "Preview complete" : "Organization complete";
      completeSummary = {
        title: summary.previewOnly
          ? `Previewed ${summary.organizedCount} files`
          : `Organized ${summary.organizedCount} files`,
        organizedCount: summary.organizedCount,
        categoryCount: summary.categoryCount,
        symlinkSkipCount: summary.symlinkSkipCount,
        undoFile: !summary.previewOnly && createUndo ? summary.undoFile : null,
      };
      resultsState = "complete";
      status = `${action}: ${summary.organizedCount} files`;
      progress = 1;
      await refreshUndoState();
    } catch (error) {
      showError(String(error));
    } finally {
      organizing = false;
    }
  }

  function requestUndo() {
    sheetState = "confirm-undo";
  }

  async function runUndo() {
    sheetState = null;
    organizing = true;
    resultsState = "loading";
    status = "Undoing organization...";
    progress = 0.1;
    completeSummary = null;
    errorMessage = "";

    try {
      const summary = await undoLastOperation();
      completeSummary = {
        title: summary.partial ? "Undo partially complete" : "Undo complete",
        organizedCount: summary.restoredCount,
        categoryCount: 0,
        symlinkSkipCount: 0,
        partial: summary.partial,
        failedCount: summary.failedCount,
      };
      resultsState = "complete";
      status = summary.partial
        ? "Undo completed with errors"
        : `Undo complete! Moved ${summary.restoredCount} files back`;
      progress = 1;
      await refreshUndoState();
    } catch (error) {
      showError(String(error));
    } finally {
      organizing = false;
    }
  }

  function closeSheet() {
    sheetState = null;
  }

  function organizeFromPreview() {
    sheetState = "confirm-organize";
  }
</script>

<div class="app">
  <aside class="config-pane">
    <header class="app-header" data-tauri-drag-region>
      <span class="app-mark" aria-hidden="true">
        <Icon name="folder" size={20} />
      </span>
      <div class="app-heading">
        <h1>OCorganize</h1>
        <p>Sort files by type, size, or date.</p>
      </div>
    </header>

    <div class="config-stack">
      <DirectoryPicker
        bind:value={directory}
        disabled={organizing}
        onchange={setDirectory}
        onerror={showError}
      />

      <StrategySelector bind:value={strategy} disabled={organizing} onchange={(value) => (strategy = value)} />

      <OptionsPanel
        bind:createUndo
        bind:previewOnly
        disabled={organizing}
        onCreateUndoChange={(value) => (createUndo = value)}
        onPreviewOnlyChange={(value) => (previewOnly = value)}
      />

      <ActionBar
        {previewOnly}
        {organizing}
        {undoEnabled}
        onPreview={runPreview}
        onOrganize={requestOrganize}
        onUndo={requestUndo}
      />
    </div>
  </aside>

  <main class="results-pane">
    <ResultsPanel
      state={resultsState}
      {status}
      {progress}
      preview={previewResult}
      complete={completeSummary}
      {errorMessage}
    />
  </main>
</div>

<Sheet open={sheetState === "preview-complete"} title="Preview complete" onclose={closeSheet}>
  {#snippet body()}
    {#if previewResult}
      <dl class="sheet-detail">
        <dt>Directory</dt>
        <dd class="sheet-path">{previewResult.directory}</dd>
        <dt>Strategy</dt>
        <dd>{previewResult.strategy}</dd>
        <dt>Files</dt>
        <dd>{previewResult.totalFiles.toLocaleString()}</dd>
        <dt>Folders</dt>
        <dd>{previewResult.categoryCount.toLocaleString()}</dd>
      </dl>
      <ul class="summary-list">
        {#each previewSummaryLines(previewResult) as line}
          <li>{line}</li>
        {/each}
      </ul>
    {/if}
  {/snippet}
  {#snippet actions()}
    <button class="btn btn-ghost" type="button" onclick={closeSheet}>Close</button>
    {#if !previewOnly}
      <button class="btn btn-primary" type="button" onclick={organizeFromPreview}>
        Organize Now
      </button>
    {/if}
  {/snippet}
</Sheet>

<Sheet open={sheetState === "confirm-organize"} title="Organize files?" onclose={closeSheet}>
  {#snippet body()}
    <p>
      Files in <strong>{directory}</strong> will be moved into category folders using the
      <strong>{strategy}</strong> strategy.
    </p>
    {#if createUndo}
      <p>An undo file will be created so you can revert this operation.</p>
    {/if}
  {/snippet}
  {#snippet actions()}
    <button class="btn btn-ghost" type="button" onclick={closeSheet}>Cancel</button>
    <button class="btn btn-primary" type="button" onclick={runOrganize}>Organize</button>
  {/snippet}
</Sheet>

<Sheet open={sheetState === "confirm-undo"} title="Undo organization?" onclose={closeSheet}>
  {#snippet body()}
    <p>This will undo the last organization and move files back to their original locations.</p>
  {/snippet}
  {#snippet actions()}
    <button class="btn btn-ghost" type="button" onclick={closeSheet}>Cancel</button>
    <button class="btn btn-danger" type="button" onclick={runUndo}>Undo</button>
  {/snippet}
</Sheet>

<style>
  .app {
    display: flex;
    height: 100%;
    overflow: hidden;
    background: var(--bg);
  }

  .config-pane {
    width: 332px;
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    border-right: 1px solid var(--border-subtle);
    background: var(--surface);
    overflow-y: auto;
  }

  .app-header {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-4) var(--space-3) var(--space-3);
  }

  .app-mark {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 38px;
    height: 38px;
    flex-shrink: 0;
    border-radius: var(--radius-md);
    background: var(--accent-soft);
    color: var(--text-primary);
    box-shadow: inset 0 0 0 1px var(--border-subtle);
  }

  .app-heading {
    min-width: 0;
  }

  .app-header h1 {
    margin: 0;
    font-size: var(--text-lg);
    font-weight: var(--weight-semibold);
    letter-spacing: -0.01em;
  }

  .app-header p {
    margin: 2px 0 0;
    font-size: var(--text-xs);
    color: var(--text-secondary);
    line-height: 1.4;
  }

  .config-stack {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    padding: var(--space-2) var(--space-3) var(--space-3);
    flex: 1;
  }

  .config-stack > :global(.section) {
    animation: section-rise var(--duration-normal) var(--ease-out) backwards;
  }

  .config-stack > :global(.section:nth-child(1)) {
    animation-delay: 40ms;
  }

  .config-stack > :global(.section:nth-child(2)) {
    animation-delay: 90ms;
  }

  .config-stack > :global(.section:nth-child(3)) {
    animation-delay: 140ms;
  }

  .config-stack > :global(.section:nth-child(4)) {
    animation-delay: 190ms;
  }

  @keyframes section-rise {
    from {
      opacity: 0;
      transform: translateY(6px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .results-pane {
    flex: 1;
    min-width: 0;
    padding: var(--space-3);
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  .summary-list {
    margin: var(--space-2) 0 0;
    padding: 0;
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .summary-list li {
    font-size: var(--text-sm);
    color: var(--text-secondary);
    padding-left: var(--space-2);
    position: relative;
  }

  .summary-list li::before {
    content: "";
    position: absolute;
    left: 2px;
    top: 8px;
    width: 4px;
    height: 4px;
    border-radius: 50%;
    background: var(--accent);
  }

  .sheet-detail {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 8px var(--space-2);
    margin: 0 0 var(--space-2);
    font-size: var(--text-sm);
  }

  .sheet-detail dt {
    color: var(--text-tertiary);
  }

  .sheet-detail dd {
    margin: 0;
    color: var(--text-primary);
    word-break: break-word;
  }

  .sheet-path {
    font-family: var(--font-mono);
    font-size: var(--text-xs);
  }

  @media (prefers-reduced-motion: reduce) {
    .config-stack > :global(.section) {
      animation: none;
    }
  }
</style>
