<script lang="ts">
  import { Button } from "$lib/components/ui/button/";
  import Moon from "@lucide/svelte/icons/moon";
  import Sun from "@lucide/svelte/icons/sun";
  import { toggleMode } from "mode-watcher";
</script>

<Button
  on:click={toggleMode}
  variant="outline"
  size="icon"
  class="absolute top-3 right-4 md:top-8 md:right-8"
>
  <Sun
    class="h-[1.2rem] w-[1.2rem] scale-100 rotate-0 transition-all dark:scale-0 dark:-rotate-90"
  />
  <Moon
    class="absolute h-[1.2rem] w-[1.2rem] scale-0 rotate-90 transition-all dark:scale-100 dark:rotate-0"
  />
  <span class="sr-only">Toggle theme</span>
</Button>
