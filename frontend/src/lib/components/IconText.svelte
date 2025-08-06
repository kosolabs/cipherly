<script lang="ts">
  import { cn } from "$lib/utils";
  import { Icon } from "@lucide/svelte";
  import type { Snippet } from "svelte";
  import type { HTMLAttributes } from "svelte/elements";

  type Props = {
    icon: typeof Icon;
    children: Snippet;
  } & HTMLAttributes<HTMLDialogElement>;

  let { children, icon: IconComponent, class: className }: Props = $props();
</script>

<div class={cn("flex items-center", className)}>
  <IconComponent class="mr-1 inline-block h-[14px] w-[14px]" />
  {@render children()}
</div>
