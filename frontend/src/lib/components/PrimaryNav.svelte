<script lang="ts">
  import { page } from "$app/stores";
  import { cn } from "$lib/utils";
  import type { Icon } from "@lucide/svelte";

  export let items: {
    href: string;
    title: string;
    disabled?: boolean;
    icon: typeof Icon;
  }[];
</script>

<div class="border-b-4 border-accent">
  <nav
    class="-mb-px flex justify-center space-x-2 md:space-x-8"
    aria-label="Tabs"
  >
    {#each items as item (item)}
      {#if item.href}
        <a
          class={cn(
            "-mb-[3px] flex items-center whitespace-nowrap border-b-4 border-transparent px-1 py-2 text-base font-semibold md:text-lg",
            $page.url.pathname !== "/" &&
              item.href.startsWith($page.url.pathname)
              ? "border-b-4 border-primary text-primary"
              : "border-accent text-muted-foreground hover:border-muted-foreground hover:text-foreground",
            item.disabled && "cursor-not-allowed opacity-80",
          )}
          href={item.disabled ? "/" : item.href}
        >
          {#if item.icon}
            <svelte:component this={item.icon} class="mr-2 h-4 w-4" />
          {/if}
          <span>{item.title}</span>
        </a>
      {/if}
    {/each}
  </nav>
</div>
