<script lang="ts">
  type IconName = "chevron" | "search" | "folder" | "warning" | "check" | "x";

  interface Props {
    name: IconName;
    size?: number;
    class?: string;
  }

  let { name, size = 16, class: className = "" }: Props = $props();
</script>

<svg
  class="icon {className}"
  width={size}
  height={size}
  viewBox="0 0 16 16"
  fill="none"
  xmlns="http://www.w3.org/2000/svg"
  aria-hidden="true"
>
  {#if name === "chevron"}
    <path
      d="M4 6L8 10L12 6"
      stroke="currentColor"
      stroke-width="1.5"
      stroke-linecap="round"
      stroke-linejoin="round"
    />
  {:else if name === "search"}
    <circle cx="7" cy="7" r="4.25" stroke="currentColor" stroke-width="1.5" />
    <path d="M10.5 10.5L13 13" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
  {:else if name === "folder"}
    <path
      d="M2 5.5C2 4.67 2.67 4 3.5 4H6.17L7.5 5.5H12.5C13.33 5.5 14 6.17 14 7V11.5C14 12.33 13.33 13 12.5 13H3.5C2.67 13 2 12.33 2 11.5V5.5Z"
      stroke="currentColor"
      stroke-width="1.5"
      stroke-linejoin="round"
    />
  {:else if name === "warning"}
    <path
      d="M8 2.5L14.5 13H1.5L8 2.5Z"
      stroke="currentColor"
      stroke-width="1.5"
      stroke-linejoin="round"
    />
    <path d="M8 6.5V9" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
    <circle cx="8" cy="11" r="0.75" fill="currentColor" />
  {:else if name === "check"}
    <path
      d="M3.5 8.5L6.5 11.5L12.5 4.5"
      stroke="currentColor"
      stroke-width="1.5"
      stroke-linecap="round"
      stroke-linejoin="round"
    />
  {:else if name === "x"}
    <path d="M4 4L12 12M12 4L4 12" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
  {/if}
</svg>

<style>
  .icon {
    display: block;
    flex-shrink: 0;
  }
</style>
