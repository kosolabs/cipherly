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
  <IconText class="mb-1 text-xs text-destructive" icon={AlertCircle}>
    {message}
  </IconText>
{/if}
