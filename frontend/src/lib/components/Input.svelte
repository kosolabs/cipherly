<script lang="ts">
  import { Input } from "$lib/components/ui/input";
  import { cn } from "$lib/utils";
  import type { HTMLInputAttributes } from "svelte/elements";

  type $$Props = HTMLInputAttributes;

  let className: $$Props["class"] = undefined;
  export let value: $$Props["value"] = undefined;
  export { className as class };
</script>

<Input
  class={cn(
    "border-2 border-muted text-base text-foreground focus:ring-0 focus-visible:ring-0",
    className,
  )}
  bind:value
  on:blur
  on:change
  on:click
  on:focus
  on:focusin
  on:focusout
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
