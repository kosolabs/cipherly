<script lang="ts">
  import { cn } from "$lib/utils";
  import { Icon } from "@lucide/svelte";
  import type { HTMLAttributes } from "svelte/elements";

  type $$Props = HTMLAttributes<HTMLDivElement> & {
    icon: typeof Icon;
  };
  let className: $$Props["class"] = undefined;

  export let icon: typeof Icon;
  export { className as class };
</script>

<div class={cn("flex items-center", className)}>
  <svelte:component this={icon} class="inline-block h-[14px] w-[14px] mr-1" />
  <slot />
</div>
