<script lang="ts">
  import { CircleAlert } from "@lucide/svelte";
  import type { ZodError } from "zod";
  import IconText from "./IconText.svelte";

  type Props = {
    path: string;
    error: ZodError | null;
  };
  let { path, error }: Props = $props();

  let message: string | undefined = $derived(
    error?.issues.find((e) => e.path.includes(path))?.message,
  );
</script>

{#if message}
  <IconText class="text-m3-error mb-1 text-xs" icon={CircleAlert}>
    {message}
  </IconText>
{/if}
