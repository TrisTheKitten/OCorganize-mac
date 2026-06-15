<script lang="ts">
  import type { Strategy } from "../api/organizer";
  import SegmentedControl from "./SegmentedControl.svelte";

  interface Props {
    value: Strategy;
    disabled?: boolean;
    onchange: (strategy: Strategy) => void;
  }

  let { value = $bindable("type"), disabled = false, onchange }: Props = $props();

  const options: { id: Strategy; label: string }[] = [
    { id: "type", label: "Type" },
    { id: "size", label: "Size" },
    { id: "date", label: "Date" },
  ];
</script>

<section class="section">
  <h3 class="section-label">Strategy</h3>
  <SegmentedControl {options} bind:value {disabled} {onchange} />
</section>
