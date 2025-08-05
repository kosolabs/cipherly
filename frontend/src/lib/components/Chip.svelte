<script lang="ts">
  import { Info } from "@lucide/svelte";
  import { Chip, Input, type InputProps } from "kosui";
  import IconText from "./IconText.svelte";

  type Props = {
    values?: string[];
    placeholder?: string;
  } & InputProps;

  let {
    values = $bindable([]),
    placeholder = "",
    ...restProps
  }: Props = $props();

  let input = $state("");

  function add(value: string) {
    values = [...new Set([...values, value])];
  }

  function remove(value: string) {
    values = values.filter((v) => v !== value);
  }

  function addAndClear() {
    if (input === "") {
      return;
    }
    add(input);
    input = "";
  }
</script>

<IconText class="text-m3-primary mb-1 text-xs" icon={Info}>
  Press Enter after each entry
</IconText>

<Input
  class="w-full"
  {placeholder}
  bind:value={input}
  variant="plain"
  onblur={addAndClear}
  onkeydown={(e) => {
    if (e.key === "Enter") {
      e.preventDefault();
      addAndClear();
    }
  }}
  {...restProps}
/>

{#if values.length > 0}
  <div class="flex flex-wrap gap-1 pt-2">
    {#each values as value (value)}
      <Chip
        class="px-3 py-1 text-sm"
        shape="circle"
        variant="tonal"
        color="secondary"
        onDelete={() => remove(value)}>{value}</Chip
      >
    {/each}
  </div>
{/if}
