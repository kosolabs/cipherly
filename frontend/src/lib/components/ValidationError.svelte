<script lang="ts">
  import { AlertCircle } from "@lucide/svelte";
  import type { ZodError } from "zod";
  import IconText from "./IconText.svelte";

  export let path: string;
  export let error: ZodError | null;

  let message: string | undefined;
  $: message = error?.issues.find((e) => e.path.includes(path))?.message;
</script>

{#if message}
  <IconText class="text-destructive mb-1 text-xs" icon={AlertCircle}>
    {message}
  </IconText>
{/if}
