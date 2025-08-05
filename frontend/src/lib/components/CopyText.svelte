<script lang="ts">
  import { Button } from "kosui";

  type Props = {
    label: string;
    text: string;
  };

  let { label, text }: Props = $props();

  let copied: boolean = $state(false);

  function copyToClipboard(text: string | null): void {
    if (!text) return;
    navigator.clipboard.writeText(text);
    copied = true;
    setTimeout(() => {
      copied = false;
    }, 500);
  }
</script>

<Button
  class="min-w-[140px]"
  type="button"
  onclick={() => copyToClipboard(text)}
>
  {#if copied}
    Copied!
  {:else}
    Copy {label}
  {/if}
</Button>
