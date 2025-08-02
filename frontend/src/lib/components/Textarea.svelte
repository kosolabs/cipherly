<script lang="ts">
  import { cn } from "$lib/utils";
  import type { HTMLTextareaAttributes } from "svelte/elements";
  import { Textarea } from "./ui/textarea";

  type $$Props = HTMLTextareaAttributes;

  let className: $$Props["class"] = undefined;
  export let value: $$Props["value"] = undefined;
  export { className as class };
</script>

<Textarea
  class={cn(
    "text-foreground border-2 text-base focus:ring-0 focus-visible:ring-0",
    className,
  )}
  bind:value
  on:blur
  on:change
  on:click
  on:focus
  on:keydown
  on:keypress
  on:keyup
  on:mouseover
  on:mouseenter
  on:mouseleave
  on:paste
  on:input
  {...$$restProps}
/>
