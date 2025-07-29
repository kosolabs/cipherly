<script lang="ts">
  import { cn } from "$lib/utils";
  import { Button as ButtonPrimitive } from "bits-ui";
  import { buttonVariants, type Props } from ".";

  type $$Props = Props;

  let className: $$Props["class"] = undefined;
  export let variant: $$Props["variant"] = "default";
  export let size: $$Props["size"] = "default";
  export let builders: $$Props["builders"] = [];
  export { className as class };

  // @ts-expect-error existing
  $: className2 = cn(buttonVariants({ variant, size, className }));
</script>

<ButtonPrimitive.Root
  {builders}
  class={cn(className2)}
  type="button"
  {...$$restProps}
  on:click
  on:keydown
>
  <slot />
</ButtonPrimitive.Root>
