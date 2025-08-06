<script lang="ts">
  import { page } from "$app/state";
  import { cn } from "$lib/utils";
  import type { Icon } from "@lucide/svelte";

  type Props = {
    items: {
      href: string;
      title: string;
      icon: typeof Icon;
    }[];
  };

  let { items }: Props = $props();
</script>

<div class="border-m3-secondary/20 border-b-4">
  <nav
    class="-mb-px flex justify-center space-x-2 md:space-x-8"
    aria-label="Tabs"
  >
    {#each items as item (item)}
      {#if item.href}
        <a
          class={cn(
            "-mb-[3px] flex items-center border-b-4 border-transparent px-1 py-2 text-base font-semibold whitespace-nowrap md:text-lg",
            page.url.pathname !== "/" && item.href.startsWith(page.url.pathname)
              ? "border-m3-primary text-m3-primary border-b-4"
              : "text-m3-secondary/80 hover:border-m3-secondary hover:text-m3-secondary",
          )}
          href={item.href}
        >
          {#if item.icon}
            <item.icon class="mr-2 h-4 w-4" />
          {/if}
          <span>{item.title}</span>
        </a>
      {/if}
    {/each}
  </nav>
</div>
