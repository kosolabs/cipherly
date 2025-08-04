<script lang="ts">
  import logo from "$lib/assets/cipherly.svg";
  import { Moon, Sun } from "@lucide/svelte";
  import { Button } from "kosui";
  import { mode, toggleMode } from "mode-watcher";
</script>

<nav class="flex items-center border-b px-4 py-2">
  <div class="flex items-center gap-2">
    <a class="flex items-center gap-2" href="/">
      <img alt="Cipherly logo" src={logo} class="size-9" />
      <div class="text-2xl font-extralight">Cipherly</div>
    </a>
  </div>
  <div class="ml-auto">
    <Button
      shape="circle"
      variant="plain"
      icon={mode.current === "light" ? Moon : Sun}
      onclick={toggleMode}
    />
  </div>
</nav>
