<script lang="ts">
  import { slide } from "svelte/transition";
  import type { PreviewCategory } from "../api/organizer";
  import { formatCount } from "../utils/format";
  import Icon from "./icons.svelte";

  interface Props {
    categories: PreviewCategory[];
  }

  let { categories }: Props = $props();

  type SortKey = "name" | "count";
  type SortDir = "asc" | "desc";

  let search = $state("");
  let sortKey = $state<SortKey>("count");
  let sortDir = $state<SortDir>("desc");
  let expanded = $state<Set<string>>(new Set());

  const filtered = $derived(
    categories.filter((category) => {
      const query = search.trim().toLowerCase();
      if (!query) {
        return true;
      }
      return (
        category.folderName.toLowerCase().includes(query) ||
        category.category.toLowerCase().includes(query) ||
        category.sampleFiles.some((file) => file.toLowerCase().includes(query))
      );
    }),
  );

  const sorted = $derived(
    [...filtered].sort((a, b) => {
      const cmp =
        sortKey === "name"
          ? a.folderName.localeCompare(b.folderName)
          : a.fileCount - b.fileCount;
      return sortDir === "asc" ? cmp : -cmp;
    }),
  );

  function toggleSort(key: SortKey) {
    if (sortKey === key) {
      sortDir = sortDir === "asc" ? "desc" : "asc";
    } else {
      sortKey = key;
      sortDir = key === "count" ? "desc" : "asc";
    }
  }

  function toggleExpanded(folderName: string) {
    const next = new Set(expanded);
    if (next.has(folderName)) {
      next.delete(folderName);
    } else {
      next.add(folderName);
    }
    expanded = next;
  }

  function sortIndicator(key: SortKey): string {
    if (sortKey !== key) {
      return "";
    }
    return sortDir === "asc" ? " ↑" : " ↓";
  }
</script>

<div class="category-table">
  <div class="search-row">
    <Icon name="search" size={14} class="search-icon" />
    <input
      type="search"
      class="search-input"
      placeholder="Search categories"
      bind:value={search}
      aria-label="Search categories"
    />
  </div>

  {#if sorted.length === 0}
    <div class="empty-search">
      <p>No categories match "{search}"</p>
    </div>
  {:else}
    <div class="table" role="table">
      <div class="table-head" role="row">
        <div
          role="columnheader"
          tabindex="0"
          class="head-cell name-cell"
          aria-sort={sortKey === "name" ? (sortDir === "asc" ? "ascending" : "descending") : "none"}
          onclick={() => toggleSort("name")}
          onkeydown={(event) => {
            if (event.key === "Enter" || event.key === " ") {
              event.preventDefault();
              toggleSort("name");
            }
          }}
        >
          Category{sortIndicator("name")}
        </div>
        <div
          role="columnheader"
          tabindex="0"
          class="head-cell count-cell"
          aria-sort={sortKey === "count" ? (sortDir === "asc" ? "ascending" : "descending") : "none"}
          onclick={() => toggleSort("count")}
          onkeydown={(event) => {
            if (event.key === "Enter" || event.key === " ") {
              event.preventDefault();
              toggleSort("count");
            }
          }}
        >
          Files{sortIndicator("count")}
        </div>
      </div>

      {#each sorted as category (category.folderName)}
        {@const isOpen = expanded.has(category.folderName)}
        {@const remaining = category.fileCount - category.sampleFiles.length}
        <div class="table-row" role="row">
          <button
            type="button"
            class="row-toggle"
            aria-expanded={isOpen}
            onclick={() => toggleExpanded(category.folderName)}
          >
            <span class="chevron" class:open={isOpen}>
              <Icon name="chevron" size={14} />
            </span>
            <span class="folder-icon">
              <Icon name="folder" size={14} />
            </span>
            <span class="folder-name">{category.folderName}/</span>
          </button>
          <span class="file-count">{formatCount(category.fileCount)}</span>
        </div>
        {#if isOpen}
          <div class="sample-files" transition:slide={{ duration: 200 }}>
            {#each category.sampleFiles as file}
              <div class="sample-file">{file}</div>
            {/each}
            {#if remaining > 0}
              <div class="sample-more">and {formatCount(remaining)} more</div>
            {/if}
          </div>
        {/if}
      {/each}
    </div>
  {/if}
</div>

<style>
  .category-table {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    min-height: 0;
  }

  .search-row {
    display: flex;
    align-items: center;
    gap: var(--space-1);
    padding: 0 var(--space-1);
    background: var(--surface-inset);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
  }

  :global(.search-icon) {
    color: var(--text-tertiary);
    margin-left: var(--space-1);
  }

  .search-input {
    flex: 1;
    border: none;
    background: transparent;
    padding: var(--space-1) var(--space-1) var(--space-1) 0;
    font-size: var(--text-sm);
    color: var(--text-primary);
    outline: none;
  }

  .search-input::placeholder {
    color: var(--text-tertiary);
  }

  .empty-search {
    padding: var(--space-6) var(--space-3);
    text-align: center;
    color: var(--text-tertiary);
    font-size: var(--text-sm);
  }

  .empty-search p {
    margin: 0;
  }

  .table {
    display: flex;
    flex-direction: column;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    overflow: hidden;
  }

  .table-head {
    display: grid;
    grid-template-columns: 1fr auto;
    background: var(--surface-inset);
    border-bottom: 1px solid var(--border-subtle);
  }

  .head-cell {
    padding: var(--space-1) var(--space-2);
    border: none;
    background: transparent;
    font-size: var(--text-xs);
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--text-tertiary);
    cursor: pointer;
    text-align: left;
  }

  .count-cell {
    text-align: right;
    min-width: 72px;
  }

  .head-cell:hover {
    color: var(--text-secondary);
  }

  .head-cell:focus-visible {
    box-shadow: inset 0 0 0 2px var(--accent-ring);
  }

  .table-row {
    display: grid;
    grid-template-columns: 1fr auto;
    align-items: center;
    border-bottom: 1px solid var(--border-subtle);
  }

  .table-row:last-child {
    border-bottom: none;
  }

  .row-toggle {
    display: flex;
    align-items: center;
    gap: var(--space-1);
    padding: 10px var(--space-2);
    border: none;
    background: transparent;
    cursor: pointer;
    text-align: left;
    min-width: 0;
    color: var(--text-primary);
    font-size: var(--text-sm);
    transition: background var(--duration-fast) var(--ease-out);
  }

  .row-toggle:hover {
    background: var(--surface-hover);
  }

  .row-toggle:focus-visible {
    box-shadow: inset 0 0 0 2px var(--accent-ring);
  }

  .chevron {
    display: flex;
    transition: transform var(--duration-fast) var(--ease-out);
    color: var(--text-tertiary);
  }

  .chevron.open {
    transform: rotate(180deg);
  }

  .folder-icon {
    color: var(--text-tertiary);
    display: flex;
  }

  .folder-name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-family: var(--font-mono);
    font-size: var(--text-sm);
  }

  .file-count {
    padding: 10px var(--space-2);
    font-size: var(--text-sm);
    font-variant-numeric: tabular-nums;
    color: var(--text-secondary);
    text-align: right;
    min-width: 72px;
  }

  .sample-files {
    padding: var(--space-1) var(--space-2) var(--space-2) calc(var(--space-2) + 38px);
    background: var(--surface-inset);
    border-bottom: 1px solid var(--border-subtle);
  }

  .sample-file,
  .sample-more {
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    color: var(--text-tertiary);
    line-height: 1.6;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .sample-more {
    font-style: italic;
    margin-top: 2px;
  }

  @media (prefers-reduced-motion: reduce) {
    .chevron {
      transition: none;
    }
  }
</style>
