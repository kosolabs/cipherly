<script lang="ts">
  import { Info, X } from "@lucide/svelte";
  import IconText from "./IconText.svelte";
  import Input from "./Input.svelte";
  import { Badge } from "./ui/badge";
  import { Button } from "./ui/button";

  export let values: string[] = [];
  export let placeholder = "";

  let input: string = "";

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

<IconText class="mb-1 text-xs text-blue-500" icon={Info}>
  Press Enter after each entry
</IconText>

<Input
  class="border-muted text-foreground border-2 text-base focus:ring-0 focus-visible:ring-0"
  {placeholder}
  bind:value={input}
  on:blur={addAndClear}
  on:keydown={(e) => {
    if (e.key === "Enter") {
      e.preventDefault();
      addAndClear();
    }
  }}
  {...$$restProps}
/>

{#if values.length > 0}
  <div class="flex flex-wrap pt-2">
    {#each values as value (value)}
      <Badge variant="secondary" class="space-x-1 text-sm">
        <span>{value}</span>
        <Button
          class="m-0 h-4 w-4 p-0 "
          variant="ghost"
          on:click={() => remove(value)}
        >
          <X class="cursor-pointer text-gray-400 hover:text-gray-500" />
        </Button>
      </Badge>
    {/each}
  </div>
{/if}
